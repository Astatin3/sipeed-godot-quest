use std::{
    sync::{
        Arc, Mutex
    },
    thread,
};

use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use ndarray::Array2;

use crate::camera::{fetch_frame::{decode_frame, frame_config_encode, normalize, ProcessedFrames}, intrinsics::{depth_to_point_cloud, DEFAULT_INTRINSICS}, PointArr};


// Constants (replace with your actual values)
const HOST: &str = "192.168.233.1";
const PORT: u16 = 80;

pub struct PrevFrames {
    pub depth: Array2<u16>,
    pub status: Array2<u16>,
}

impl Default for PrevFrames {
    fn default() -> Self {
        Self {
            depth: Array2::from_elem((240, 320), 0),
            status: Array2::from_elem((240, 320), 0),
        }
    }
}

pub struct SipeedCamera {
    frames: Arc<Mutex<ProcessedFrames>>,
    prev_frames: PrevFrames,
    thread_handle: Option<thread::JoinHandle<()>>,
    // point_cloud: LivePointView,
}

impl Default for SipeedCamera {
    fn default() -> Self {
        // Shared state for the latest processed frames
        let frames = Arc::new(Mutex::new(ProcessedFrames::default()));

        let frames_clone = Arc::clone(&frames);
        let decoder_handle = thread::spawn(move || {
            loop {
                match fetch_frame() {
                    Ok(frame_data) => match decode_frame(&frame_data) {
                        Ok(processed) => {
                            *frames_clone.lock().unwrap() = processed;
                        }
                        Err(e) => warn!("Error decoding frame: {}", e),
                    },
                    Err(e) => warn!("Error fetching frame: {}", e),
                }
            }
        });

        Self {
            frames,
            thread_handle: Some(decoder_handle),
            prev_frames: PrevFrames::default(),
            // point_cloud: LivePointView::default(),
        }
    }
}

impl SipeedCamera {
    pub fn get_points(&mut self) -> Option<PointArr> {
        let frames_lock = self.frames.lock().unwrap();
        let ref frames = *frames_lock;

        // let prev_status_lock = self.prev_status.lock();
        // let ref prev_status = *prev_status_lock;

        let mut processed = false;

        // Display depth image
        if let Some(ref depth) = frames.depth {
            self.prev_frames.depth = (&self.prev_frames.depth + depth) / 2;

            let depth_viz = normalize(&self.prev_frames.depth);
            processed = true;
        }

        // // Display IR image
        // if let Some(ref ir) = frames.ir {
        //     let ir_viz = normalize(ir);
        //     ui.heading("IR");
        //     image_widget(ui, "ir_img", &ir_viz, [320.0, 240.0]);
        //     processed = true;
        // }

        // Display status image if available
        if let Some(ref status) = frames.status {
            self.prev_frames.status = (&self.prev_frames.status + (2 - status)) / 2;
            let status_viz = normalize(&self.prev_frames.status);
            processed = true;
        }

        // Display RGB image if available
        if let Some(ref rgb) = frames.rgb {
            let rgb_viz = rgb.as_slice().unwrap();
            let size = match rgb.dim().1 {
                640 => [640.0, 480.0],
                800 => [800.0, 600.0],
                _ => [640.0, 480.0], // Default
            };
            processed = true;
        }

        if let Some(ref status) = frames.status {
            if let Some(ref rgb) = frames.rgb {
                let mut points: PointArr = Vec::new();

                for i in 0..(320 * 240) {
                    let x = i / 240;
                    let y = i % 320;

                    // println!("{:?}", status.get((x, y)));

                    let status = status.get((x, y));

                    if status.is_none() || *status.unwrap() != 0 {
                        continue;
                    }

                    // let prev_status = prev_status.get((x, y));

                    // if prev_status.is_none() || *prev_status.unwrap() != 0 {
                    //     continue;
                    // }

                    // if *status.get((x, y)).unwrap() != 0u16 {
                    //     continue;
                    // }
                    //

                    let (r, g, b) = if let Some((rgbx, rgby)) = scale_shift_rgb_xy(x, y) {
                        (
                            *rgb.get((rgbx, rgby, 0)).unwrap() as u8,
                            *rgb.get((rgbx, rgby, 1)).unwrap() as u8,
                            *rgb.get((rgbx, rgby, 2)).unwrap() as u8,
                        )
                    } else {
                        (255, 255, 255)
                    };

                    let d = self.prev_frames.depth.get((x, y)).unwrap();

                    let (x, y, z) = depth_to_point_cloud(
                        x as i32,
                        y as i32,
                        d + 50,
                        &DEFAULT_INTRINSICS,
                    );

                    points.push((x, y, z, r, g, b))
                }

                return Some(points)
            }
        }

        None

        // self.point_cloud.render(ui);
    }
}

fn scale_shift_rgb_xy(x: usize, y: usize) -> Option<(usize, usize)> {
    static X_OFFSET: f32 = 30.;
    static Y_OFFSET: f32 = 22.;
    static SCAME: f32 = 1.75;

    let x = ((x as f32 + X_OFFSET) * SCAME) as usize;
    let y = ((y as f32 + Y_OFFSET) * SCAME) as usize;

    if y >= 640 {
        return None;
    }
    if x >= 480 {
        return None;
    }

    // println!("{}, {}", x, y);

    Some((x as usize, y as usize))
}

fn fetch_frame() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    is_success(&frame_config_encode(1, 0, 255, 0, 2, 7, 1, 0, 0))?;


    let url = format!("http://{}:{}/getdeep", HOST, PORT);

    trace!("Fetching images from: {}", url);

    let response = ureq::get(url).call()?;

    if response.status() != 200 {
        return Err(format!("Failed to get frame: HTTP {}", response.status()).into());
    }

    trace!("Got deep image");
    let deep_img = response.into_body().read_to_vec()?;
    trace!("Length={}", deep_img.len());

    // Parse frame ID and timestamp
    if deep_img.len() >= 16 {
        let mut cursor = Cursor::new(&deep_img[0..16]);
        let frame_id = cursor.read_u64::<LittleEndian>()?;
        let stamp_msec = cursor.read_u64::<LittleEndian>()?;
        trace!(
            "Frame ID: {}, Timestamp: {:.3}s",
            frame_id,
            stamp_msec as f64 / 1000.0
        );
    }

    return Ok(deep_img);
}

fn is_success(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}/set_cfg", HOST, PORT);

    trace!("Sending request to: {}", url);

    let response = ureq::post(url).send(data.to_vec())?;
    if response.status() == 200 {
        return Ok(());
    } else {
        return Err(format!("Status code: {}", response.status().to_string()).into());
    }
}

// Helper to display images in egui
// fn image_widget(ui: &mut egui::Ui, id: &str, rgb_data: &[u8], size: [f32; 2]) {
//     let color_image = egui::ColorImage::from_rgb([size[0] as usize, size[1] as usize], rgb_data);

//     let handle = ui
//         .ctx()
//         .load_texture(id, color_image, egui::TextureOptions::LINEAR);

//     let sized_image =
//         egui::load::SizedTexture::new(handle.id(), egui::vec2(size[0] as f32, size[1] as f32));

//     ui.image(sized_image);
// }
