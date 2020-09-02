use crate::film::Pixel;
use crate::integrator::{Integrator, IntegratorManager};
use crate::light::light_picker::{self, LightPicker};
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::shading::lobe::LobeType;
use crate::shading::material::{MaterialPool, ShadingCoord};
use crate::spectrum::Color;
use pmath::ray::{PrimaryRay, Ray};

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
        let mut color_result = Color::black();
        let mut throughput = Color::white();
        let mut ray = prim_ray.ray;

        // Whether or not we had a specular bounce just now
        let mut specular_bounce = false;

        for bounce_count in 0..self.max_bounce {
            let interaction = match scene.intersect(ray) {
                Some(int) => int,
                None => break,
            };

            // Get the bsdf and updated interaction:
            let (bsdf, interaction) = materials
                .get_material(interaction.material_id)
                .bsdf(interaction);

            // Sample the light(s):
            color_result += throughput
                * light_picker::sample_lights(
                    interaction,
                    bsdf,
                    ray.time,
                    scene,
                    sampler,
                    light_picker,
                );

            // Sample the bsdf for the next ray:
            let shading_coord = ShadingCoord::new(interaction);
            let (bsdf_color, wi, bsdf_pdf, lobe_type) =
                bsdf.sample(-ray.dir, sampler.sample(), LobeType::ALL, shading_coord);

            if bsdf_color.is_black() || (bsdf_pdf == 0.0) {
                break;
            }

            throughput = (throughput * bsdf_color * wi.dot(interaction.shading_n).abs())
                .scale(1.0 / bsdf_pdf);
            specular_bounce = lobe_type.contains(LobeType::SPECULAR);
            ray = Ray::new(interaction.p, wi, ray.time);
        }

        Pixel::add_sample(color_result)
    }
}
