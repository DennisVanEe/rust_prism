use pmath::vector::{Vec2, Vec3};

/// Represents any information that we may need for
#[derive(Clone, Copy, Debug)]
pub struct GeomSurf {
    uv: Vec2<f64>,   // uv coordinate at the intersection
    dpdu: Vec3<f64>, // vectors parallel to the triangle
    dpdv: Vec3<f64>,

    sn: Vec3<f64>,    // the shading normal at this point
    sdpdu: Vec3<f64>, // the shading dpdu, dpdv at this point
    sdpdv: Vec3<f64>,
    sdndu: Vec3<f64>, // the shading dndu, dndv at this point
    sdndv: Vec3<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct VolSurf {}

#[derive(Clone, Copy, Debug)]
pub enum SurfType {
    Geom(GeomSurf),
    Vol(VolSurf),
}

#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub p: Vec3<f64>,  // intersection point
    pub n: Vec3<f64>,  // geometric normal (of triangle)
    pub wo: Vec3<f64>, // direction of intersection leaving the point
    pub t: f64,        // the parametric parameter of the ray where the intersection happened
    pub time: f64,     // the time period when the intersection happened

    pub surf_type: SurfType, // the type of interaction where the intersection occurs
}
