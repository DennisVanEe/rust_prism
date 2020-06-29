use crate::camera::{Camera, CameraSample};
use crate::math::bbox::BBox2;
use crate::math::matrix::Mat4;
use crate::math::ray::{PrimaryRay, Ray, RayDiff};
use crate::math::sampling;
use crate::math::vector::{Vec2, Vec3};
use crate::transform::Transf;

// NOTE: camera space has the camera at the origin, the y-axis pointing up,
// the x-axis pointing right, and the z-axis pointing into the screen.

#[derive(Clone, Copy, Debug)]
pub struct PerspectiveCamera {
    // Defines the position of the camera in the world
    camera_to_world: Transf,
    camera_to_screen: Mat4<f64>,
    raster_to_camera: Mat4<f64>,
    screen_to_raster: Mat4<f64>,
    raster_to_screen: Mat4<f64>,

    lens_radius: f64,
    focal_dist: f64,

    // Cached these values for efficient ray diff generation:
    dx_camera: Vec3<f64>,
    dy_camera: Vec3<f64>,
}

impl PerspectiveCamera {
    /// Constructs a new Perspective Camera.
    ///
    /// # Arguments
    /// * `camera_to_world` - Camera to world transform. Defines place in world.
    /// * `fov` - The field-of-view of the camera (in degrees)
    /// * `shutter_open` - The time to open the shutter of the camera
    /// * `shutter_close` - The time to close the shutter of the camera
    /// * `lens_readius` - The radius of the lens in use
    /// * `focal_dist` - The focal distance of the lense
    /// * `screen_window` - The size of the "sesnor"
    /// * `pixel_res` - The resolution of the camera
    pub fn new(
        camera_to_world: Transf,
        fov: f64,
        shutter_open: f64,
        shutter_close: f64,
        lens_radius: f64,
        focal_dist: f64,
        screen_window: BBox2<f64>,
        pixel_res: Vec2<usize>,
    ) -> Self {
        // Projects a point in camera space to screen space.
        let camera_to_screen = Mat4::new_perspective(fov, 1e-2, 1000.0);

        // Then, finally, we scale it by the pixel resolution so it's on a specific pixel.
        let screen_to_raster = Mat4::new_scale(Vec3 {
            x: pixel_res.x as f64,
            y: pixel_res.y as f64,
            z: 1.,
            // Then we scale the point by the inverse of the screen's dimensions so that
            // it's in NDC space (normal device coordinate space).
        }) * Mat4::new_scale(Vec3 {
            x: 1. / (screen_window.pmax.x - screen_window.pmin.x),
            y: 1. / (screen_window.pmin.y - screen_window.pmax.y),
            z: 1.,
            // First translate a point in screen space so that the origin is at the top-left corner
            // of the screen (this means translating by the bottom-left corner)
        }) * Mat4::new_translate(Vec3 {
            x: -screen_window.pmin.x,
            y: -screen_window.pmax.y,
            z: 0.,
        });
        let raster_to_screen = screen_to_raster.inverse();
        let raster_to_camera = camera_to_screen.inverse() * raster_to_screen;

        // Calculate these values to cache:
        let dx_camera = raster_to_camera.mul_vec_proj(Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }) - raster_to_camera.mul_vec_proj(Vec3::zero());
        let dy_camera = raster_to_camera.mul_vec_proj(Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }) - raster_to_camera.mul_vec_proj(Vec3::zero());

        PerspectiveCamera {
            camera_to_world,
            camera_to_screen,
            raster_to_camera,
            screen_to_raster,
            raster_to_screen,
            lens_radius,
            focal_dist,
            dx_camera,
            dy_camera,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn gen_ray(&self, sample: CameraSample) -> Ray<f64> {
        // Camera point:
        let p_camera = self
            .raster_to_camera
            .mul_vec_proj(Vec3::from_vec2(sample.p_film, 0.0));
        // Calculate a ray from the origin of Camera space to the point on the camera:
        let ray = Ray::new(Vec3::zero(), p_camera.normalize(), sample.time);

        // Check if there is a lens and, so, update the ray if that is the case:
        let ray = if self.lens_radius > 0.0 {
            let p_lens = sampling::concentric_sample_disk(sample.p_lens).scale(self.lens_radius);
            // The point on the place of focus:
            let ft = self.focal_dist / ray.dir.z;
            let p_focus = ray.point_at(ft);
            Ray::new(
                Vec3::from_vec2(p_lens, 0.),
                (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize(),
                sample.time,
            )
        } else {
            ray
        };

        self.camera_to_world.ray(ray)
    }

    fn gen_primary_ray(&self, sample: CameraSample) -> PrimaryRay<f64> {
        // return PrimaryRay {
        //     ray: Ray::new(Vec3 {x: -2.0, y: 0.0, z: 0.0 }, Vec3 { x: 1.0, y: 0.0, z: 0.0 }, 1.0),
        //     ray_diff: RayDiff {
        //         rx_org: Vec3::zero(),
        //         ry_org: Vec3::zero(),
        //         rx_dir: Vec3::zero(),
        //         ry_dir: Vec3::zero(),
        //     }
        // };

        // Camera point:
        let p_camera = self
            .raster_to_camera
            .mul_vec_proj(Vec3::from_vec2(sample.p_film, 0.0));
        // Calculate a ray from the origin of Camera space to the point on the camera:
        let ray = Ray::new(Vec3::zero(), p_camera.normalize(), sample.time);

        // Check whether or not there is a lens
        let prim_ray = if self.lens_radius > 0. {
            // Calculate the focus information as normal:
            let p_lens = sampling::concentric_sample_disk(sample.p_lens).scale(self.lens_radius);

            let ft = self.focal_dist / ray.dir.z;
            let p_focus = ray.point_at(ft);
            let ray = Ray::new(
                Vec3::from_vec2(p_lens, 0.),
                (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize(),
                sample.time,
            );

            // Calculate the focus information in the dx direction:
            let dir_x = (p_camera + self.dx_camera).normalize();
            let ft = self.focal_dist / dir_x.z;
            let p_focus = dir_x.scale(ft); // + (0,0,0) as it stems from the origin
            let rx_org = Vec3::from_vec2(p_lens, 0.);
            let rx_dir = (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize();

            // Calculate the focus information in the dy direction:
            let dir_y = (p_camera + self.dy_camera).normalize();
            let ft = self.focal_dist / dir_y.z;
            let p_focus = dir_y.scale(ft); // + (0,0,0) as it stems from the origin
            let ry_org = Vec3::from_vec2(p_lens, 0.);
            let ry_dir = (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize();

            PrimaryRay {
                ray,
                ray_diff: RayDiff {
                    rx_org,
                    rx_dir,
                    ry_org,
                    ry_dir,
                },
            }
        } else {
            // No lens to deal with, so this shouldn't be too hard to create
            // the extra rx and ry values:

            PrimaryRay {
                ray,
                ray_diff: RayDiff {
                    rx_org: ray.org,
                    rx_dir: (p_camera + self.dx_camera).normalize(),
                    ry_org: ray.org,
                    ry_dir: (p_camera + self.dy_camera).normalize(),
                },
            }
        };

        // TODO: more elegant solution to the normalization thing

        // Don't forget to transform it back to world space!
        let mut prim_ray = self.camera_to_world.primary_ray(prim_ray);
        prim_ray.ray.dir = prim_ray.ray.dir.normalize();
        prim_ray
    }
}
