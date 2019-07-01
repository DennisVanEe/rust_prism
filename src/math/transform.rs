use super::matrix::Mat4;
/// The transform is what get's exposed to the user when they want
/// to transform objects. Currently, the transform only works
/// with 32 bit floating point values.
use super::vector::Vec3;

/// Transforms, being a massive 32 floats, can't be copied
/// willy nilly. To do that you have to clone it.
/// Transforms are always gauranteed to be invertible.
#[derive(Clone, Debug)]
pub struct Transform {
    nrm: Mat4<f32>,
    inv: Mat4<f32>,
}

impl Transform {
    /// The function will perform the inversion itself.
    /// Note that, because the inverse can be undefined,
    /// it returns an optional.
    pub fn new(nrm: Mat4<f32>) -> Option<Transform> {
        let invOpt = nrm.inverse();
        match invOpt {
            Some(inv) => Some(Transform { nrm, inv }),
            None => None,
        }
    }

    // generates translation for transform:
    pub fn translation(trans: Vec3<f32>) -> Transform {
        let nrm = Mat4::translation(trans);
        let inv = Mat4::translation(-trans);
        Transform { nrm, inv }
    }

    // generates the inverse of the transformation (just the old swap):
    pub fn inverse(&self) -> Transform {
        let matCopy = self.clone();
        Transform {
            nrm: matCopy.inv,
            inv: matCopy.nrm,
        }
    }
}
