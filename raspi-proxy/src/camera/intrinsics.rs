use std::f64;

pub static DEFAULT_INTRINSICS: CameraIntrinsics = CameraIntrinsics {
    fx: 2.318290e+02,
    fy: 2.327785e+02,
    u0: 1.669372e+02,
    v0: 1.235151e+02,
    k1: 5.857900e-02,
    k2: 2.431399e-02,
    k3: -2.737180e-01,
    p1: 2.299994e-05,
    p2: 1.658998e-03,
};

/// Camera intrinsic parameters
#[derive(Debug, Clone, Copy)]
pub struct CameraIntrinsics {
    pub fx: f64, // focal length x
    pub fy: f64, // focal length y
    pub u0: f64, // principal point x
    pub v0: f64, // principal point y
    pub k1: f64, // radial distortion coefficient 1
    pub k2: f64, // radial distortion coefficient 2
    pub k3: f64, // radial distortion coefficient 3
    pub p1: f64, // tangential distortion coefficient 1
    pub p2: f64, // tangential distortion coefficient 2
}

impl CameraIntrinsics {
    pub fn new(
        fx: f64,
        fy: f64,
        u0: f64,
        v0: f64,
        k1: f64,
        k2: f64,
        k3: f64,
        p1: f64,
        p2: f64,
    ) -> Self {
        Self {
            fx,
            fy,
            u0,
            v0,
            k1,
            k2,
            k3,
            p1,
            p2,
        }
    }
}

/// Convert a pixel with depth information to a 3D point
///
/// # Arguments
/// * `x` - x-coordinate in the image (columns)
/// * `y` - y-coordinate in the image (rows)
/// * `depth` - depth value at the pixel (typically in millimeters)
/// * `intrinsics` - camera intrinsic parameters
///
/// # Returns
/// A 3D point in the camera coordinate system
pub fn depth_to_point_cloud(
    x: i32,
    y: i32,
    depth: u16,
    intrinsics: &CameraIntrinsics,
) -> (i32, i32, i32) {
    // Convert pixel coordinates to normalized image coordinates
    let x_f = x as f64;
    let y_f = y as f64;

    // Apply distortion correction if needed
    let (x_corrected, y_corrected) = correct_distortion(x_f, y_f, intrinsics);

    // Convert normalized image coordinates to camera coordinates
    let z = depth as f64; // depth in camera z-direction

    // Back-project using the pinhole camera model
    // (x - u0) / fx = X / Z
    // (y - v0) / fy = Y / Z
    let x_3d = (x_corrected - intrinsics.u0) * z / intrinsics.fx;
    let y_3d = (y_corrected - intrinsics.v0) * z / intrinsics.fy;

    // Return the 3D point, converting to integer values if needed
    (x_3d.round() as i32, y_3d.round() as i32, z.round() as i32)
}

/// Corrects for lens distortion based on the Brown-Conrady model
///
/// # Arguments
/// * `x` - uncorrected x coordinate in pixel space
/// * `y` - uncorrected y coordinate in pixel space
/// * `intrinsics` - camera intrinsic parameters with distortion coefficients
///
/// # Returns
/// Corrected (x, y) coordinates
fn correct_distortion(x: f64, y: f64, intrinsics: &CameraIntrinsics) -> (f64, f64) {
    // Convert to normalized image coordinates (centered at principal point)
    let x_norm = (x - intrinsics.u0) / intrinsics.fx;
    let y_norm = (y - intrinsics.v0) / intrinsics.fy;

    // Calculate squared radius for radial distortion
    let r2 = x_norm * x_norm + y_norm * y_norm;
    let r4 = r2 * r2;
    let r6 = r4 * r2;

    // Calculate radial distortion factor
    let radial_factor = 1.0 + intrinsics.k1 * r2 + intrinsics.k2 * r4 + intrinsics.k3 * r6;

    // Calculate tangential distortion
    let tangential_x =
        2.0 * intrinsics.p1 * x_norm * y_norm + intrinsics.p2 * (r2 + 2.0 * x_norm * x_norm);
    let tangential_y =
        intrinsics.p1 * (r2 + 2.0 * y_norm * y_norm) + 2.0 * intrinsics.p2 * x_norm * y_norm;

    // Apply distortion correction
    let x_corrected = x_norm * radial_factor + tangential_x;
    let y_corrected = y_norm * radial_factor + tangential_y;

    // Convert back to pixel coordinates
    let x_pixel = x_corrected * intrinsics.fx + intrinsics.u0;
    let y_pixel = y_corrected * intrinsics.fy + intrinsics.v0;

    (x_pixel, y_pixel)
}
