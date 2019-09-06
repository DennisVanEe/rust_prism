use crate::transform::{StaticTransform, Transform};

pub struct PerspectiveCamera<T: Transform> {
    camera_to_world: T, // defines the position of the camera in the world
    camera_to_screen: StaticTransform,
    raster_to_camera: StaticTransform,
    screen_to_raster: StaticTransform,
    raster_to_screen: StaticTransform,
    
}