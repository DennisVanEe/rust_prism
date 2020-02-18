use crate::camera::{Camera, CameraSample};
use crate::film::Film;
use crate::pixel_buffer::TileOrdering;
use crate::math::bbox::BBox2;
use crate::math::ray::{Ray, RayDiff};
use crate::math::sampling;
use crate::math::vector::Vec3;
use crate::transform::{StaticTransform, Transform};

pub struct PerspectiveCamera<T: Transform, O: TileOrdering> {
    // Defines the position of the camera in the world
    camera_to_world: T,
    camera_to_screen: StaticTransform,
    raster_to_camera: StaticTransform,
    screen_to_raster: StaticTransform,
    raster_to_screen: StaticTransform,

    lens_radius: f64,
    focal_dist: f64,

    // Cached these values for efficient generation:
    dx_camera: Vec3<f64>,
    dy_camera: Vec3<f64>,

    // Film information that the camera uses. The camera is the
    // owner of the film in this case:
    film: Film<O>,
}

impl<T: Transform, O: TileOrdering> PerspectiveCamera<T, O> {
    pub fn new(
        camera_to_world: T,
        camera_to_screen: StaticTransform,
        shutter_open: f64,
        shutter_close: f64,
        lens_radius: f64,
        focal_dist: f64,
        screen_window: BBox2<f64>,
        film: Film<O>,
    ) -> Self {
        let pixel_res = film.get_pixel_res();
        let screen_to_raster = StaticTransform::new_scale(Vec3 {
            x: pixel_res.x as f64,
            y: pixel_res.y as f64,
            z: 1.,
        }) * StaticTransform::new_scale(Vec3 {
            x: 1. / (screen_window.pmax.x - screen_window.pmin.x),
            y: 1. / (screen_window.pmin.y - screen_window.pmax.y),
            z: 1.,
        }) * StaticTransform::new_translate(Vec3 {
            x: -screen_window.pmin.x,
            y: -screen_window.pmax.y,
            z: 0.,
        });
        let raster_to_screen = screen_to_raster.inverse();
        let raster_to_camera = camera_to_screen.inverse() * raster_to_screen;

        // Calculate these values to cache:
        let dx_camera = raster_to_camera.proj_point(Vec3 {
            x: 1.,
            y: 0.,
            z: 0.,
        }) - raster_to_camera.proj_point(Vec3::zero());
        let dy_camera = raster_to_camera.proj_point(Vec3 {
            x: 0.,
            y: 1.,
            z: 0.,
        }) - raster_to_camera.proj_point(Vec3::zero());

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
            film,
        }
    }
}

impl<T: Transform, O: TileOrdering> Camera for PerspectiveCamera<T, O> {
    fn generate_ray(&self, sample: CameraSample) -> Ray<f64> {
        let p_camera = self
            .raster_to_camera
            .proj_point(Vec3::from_vec2(sample.p_film, 0.));
        let ray = Ray {
            org: Vec3::zero(),
            dir: p_camera.normalize(),
        };

        // Check if there is a lens and, so, update the ray if that is the case:
        let ray = if self.lens_radius > 0. {
            let p_lens = sampling::concentric_sample_disk(sample.p_lens).scale(self.lens_radius);
            // The point on the place of focus:
            let ft = self.focal_dist / ray.dir.z;
            let p_focus = ray.point_at(ft);
            Ray {
                org: Vec3::from_vec2(p_lens, 0.),
                dir: (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize(),
            }
        } else {
            ray
        };

        let camera_to_world_int = self.camera_to_world.interpolate(sample.time);
        camera_to_world_int.ray(ray)
    }

    fn generate_raydiff(&self, sample: CameraSample) -> (Ray<f64>, RayDiff<f64>) {
        let p_camera = self
            .raster_to_camera
            .proj_point(Vec3::from_vec2(sample.p_film, 0.));
        let ray = Ray {
            org: Vec3::zero(),
            dir: p_camera.normalize(),
        };

        // Check whether or not there is a lens
        if self.lens_radius > 0. {
            // Calculate the focus information as normal:
            let p_lens = sampling::concentric_sample_disk(sample.p_lens).scale(self.lens_radius);

            let ft = self.focal_dist / ray.dir.z;
            let p_focus = ray.point_at(ft);
            let ray = Ray {
                org: Vec3::from_vec2(p_lens, 0.),
                dir: (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize(),
            };

            // Calculate the focus information in the dx direction:
            let dir_x = (p_camera + self.dx_camera).normalize();
            let ft = self.focal_dist / dir_x.z;
            let p_focus = dir_x.scale(ft); // + (0,0,0) as it stems from the origin
            let rx = Ray {
                org: Vec3::from_vec2(p_lens, 0.),
                dir: (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize(),
            };
            // Calculate the focus information in the dy direction:
            let dir_y = (p_camera + self.dy_camera).normalize();
            let ft = self.focal_dist / dir_y.z;
            let p_focus = dir_y.scale(ft); // + (0,0,0) as it stems from the origin
            let ry = Ray {
                org: Vec3::from_vec2(p_lens, 0.),
                dir: (p_focus - Vec3::from_vec2(p_lens, 0.)).normalize(),
            };

            (ray, RayDiff { rx, ry })
        } else {
            // No lens to deal with, so this shouldn't be too hard to create
            // the extra rx and ry values:
            let rx = Ray {
                org: ray.org,
                dir: (p_camera + self.dx_camera).normalize(),
            };
            let ry = Ray {
                org: ray.org,
                dir: (p_camera + self.dy_camera).normalize(),
            };

            (ray, RayDiff { rx, ry })
        }
    }
}
