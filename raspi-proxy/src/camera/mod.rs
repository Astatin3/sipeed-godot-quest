mod fetch_frame;
mod intrinsics;
mod camera;

pub use camera::SipeedCamera;

pub type Point = (i32, i32, i32, u8, u8, u8);
pub type PointArr = Vec<Point>;