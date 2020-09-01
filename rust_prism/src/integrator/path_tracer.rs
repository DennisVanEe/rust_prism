use crate::film::Pixel;
use crate::integrator::{Integrator, IntegratorManager};
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Color;
use pmath::ray::PrimaryRay;
use pmath::vector::Vec3;

pub struct PathTracerIntegratorManager {
    max_bounce: u32,
}

impl PathTracerIntegratorManager {
    pub fn new(max_bounce: u32) -> Self {
        PathTracerIntegratorManager { max_bounce }
    }
}

impl IntegratorManager<PathTracerIntegrator> for PathTracerIntegratorManager {
    fn spawn_integrator(&self, _thread_id: u32) -> PathTracerIntegrator {
        PathTracerIntegrator {
            max_bounce: self.max_bounce,
        }
    }
}

pub struct PathTracerIntegrator {
    max_bounce: u32,
}

impl Integrator for PathTracerIntegrator {
    fn integrate(
        &mut self,
        prim_ray: PrimaryRay<f64>,
        scene: &Scene,
        _sampler: &mut Sampler,
        pixel: Pixel,
    ) -> Pixel {
        let mut throughput = Color::white();
        let mut ray = prim_ray.ray;

        // Whether or not we had a specular bounce just now
        let mut specular_bounce = false;

        for bounce_count in 0..self.max_bounce {
            let interaction = match scene.intersect(ray) {
                Some(int) => int,
                None => break,
            };
        }
    }
}
