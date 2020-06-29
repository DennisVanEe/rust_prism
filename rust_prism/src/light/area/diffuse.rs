// use super::AreaLight;
// use crate::geometry::}Interaction, Geometry};
// use crate::spectrum::Spectrum;

// // TODO:
// // current implementation seems sort of shoddy and hacky. I want
// // the geometry and light to be more intwined. As if there is a function
// // that you can call to intersect a light and get's it's readiance from a specific direction
// // in one step. Hmmm, let's think about this for a bit.

// // For now the geometry attached to this type will be a generic.
// // This could potentially get out of hand if we have a lot of geometry
// // but seems best for now:
// pub struct DiffuseLight<G: Geometry> {
//     // The geometry of the light:
//     geometry: G,
//     // The base spectrum of the DiffuseLight
//     radiance: Spectrum,
// }

// impl AreaLight for DiffuseLight {
//     fn eval(int: Interaction, w: Vec3<f64>) -> Spectrum {
//         // We simply check if the light is on the same side as the normal:

//     }
// }
