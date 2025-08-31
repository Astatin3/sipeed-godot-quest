#[macro_use]
extern crate log;
mod camera;

use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

use camera::SipeedCamera;

const SOCKET: &'static str = "0.0.0.0:1234";

enum DataBlocks {
    Error = 0,
    PointCloudData = 1,
    ReadyData = 2
}

pub fn main() {
    pretty_env_logger::init();

    let mut camera = SipeedCamera::default();

    info!("Connection established with camera");

    // loop {
    //     let points = camera.get_points();
    //     info!("{:?}", points);
    // }


    let mut listener = TcpListener::bind(SOCKET).expect("Failed to bind");

    info!("Server listening on {}", SOCKET);

    loop {
        match run_server(&mut camera, &mut listener) {
            Err(e) => {
                error!("{}", e);
            }
            _ => {}
        }
    }

    // println!("Test!");
}

fn run_server(camera: &mut SipeedCamera, listener: &mut TcpListener) -> Result<(), std::io::Error> {
    for stream in listener.incoming() {
        let mut stream = stream?;

        info!("New connection: {}", stream.peer_addr().unwrap());
        run_stream(camera, &mut stream)?;

    }

    Ok(())
}

fn run_stream(camera: &mut SipeedCamera,stream: &mut TcpStream) ->  Result<(), std::io::Error> {
    loop {
        let mut recv_buf = [0u8; 1];
        stream.read_exact(&mut recv_buf)?;
        assert!(recv_buf[0] == DataBlocks::ReadyData as u8);


        let mut bytes = Vec::new();

        let points = camera.get_points();

        if let Some(points) = points {


            // bytes.append(&mut ("A").as_bytes().to_vec());
            // bytes.append(&mut "B".as_bytes().to_vec());

            bytes.append(&mut (DataBlocks::PointCloudData as i32).to_le_bytes().to_vec());
            bytes.append(&mut (points.len() as i32).to_le_bytes().to_vec());
            // bytes.append(&mut points.len().to_le_bytes().to_vec());

            for point in points {
                bytes.append(&mut point.0.to_le_bytes().to_vec());
                bytes.append(&mut point.1.to_le_bytes().to_vec());
                bytes.append(&mut point.2.to_le_bytes().to_vec());
                bytes.append(&mut point.3.to_le_bytes().to_vec());
                bytes.append(&mut point.4.to_le_bytes().to_vec());
                bytes.append(&mut point.5.to_le_bytes().to_vec());
            }

        } else {
            bytes.append(&mut (DataBlocks::Error as i32).to_le_bytes().to_vec());
        }



        stream.write_all(&bytes)?;
        stream.flush()?;

        info!("Sent {} bytes", bytes.len());
    }
}