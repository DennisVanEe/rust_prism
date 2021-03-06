use crate::film::Pixel;
use crate::integrator::{Integrator, IntegratorManager};
use crate::light::light_picker::LightPicker;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::shading::material::MaterialPool;
use crate::spectrum::Color;
use pmath::ray::PrimaryRay;
use pmath::vector::Vec3;

pub struct NormalIntegratorManager {
    use_geom_normal: bool,
}

impl NormalIntegratorManager {
    pub fn new(param: bool) -> Self {
        NormalIntegratorManager {
            use_geom_normal: param,
        }
    }
}

impl IntegratorManager<NormalIntegrator> for NormalIntegratorManager {
    fn spawn_integrator(&self, _thread_id: u32) -> NormalIntegrator {
        NormalIntegrator {
            use_geom_normal: self.use_geom_normal,
        }
    }
}

/// A simple integrator that just returns the scene space normals.
pub struct NormalIntegrator {
    use_geom_normal: bool, // Whether or not to use geometric or shading normals
}

impl Integrator for NormalIntegrator {
    fn integrate<LI, L>(
        &mut self,
        prim_ray: PrimaryRay<f64>,
        scene: &Scene,
        materials: &MaterialPool,
        light_picker: &L,
        sampler: &mut Sampler,
        pixel: Pixel,
    ) -> Pixel
    where
        LI: Iterator<Item = (u32, f64)>,
        L: LightPicker<LI>,
    {
        // Intersect the scene and get the normal at the intersection.
        let normal = match scene.intersect(prim_ray.ray) {
            Some(int) => {
                let normal = if self.use_geom_normal {
                    int.n
                } else {
                    int.shading_n
                };
                // We need the range to be between 0 and 1 (no hdr here).
                (Vec3::one() + normal).scale(0.5)
            }
            _ => Vec3::zero(),
        };

        // Add them to the pixel
        pixel.add_sample(Color::from_vec3(normal))
    }
}
