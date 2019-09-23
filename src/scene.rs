use crate::geometry::{Geometry, Interaction};
use crate::math::bbox::BBox3;
use crate::math::ray::Ray;
use crate::shading::material::{Bsdf, Material};
use crate::transform::Transform;

use bumpalo::Bump;
use simple_error::{bail, SimpleResult};

use std::collections::HashMap;
use std::mem::{transmute, ManuallyDrop};

struct Model<'a> {
    pub geometry: &'a dyn Geometry,
    pub material: &'a dyn Material,
    pub transform: &'a dyn Transform,
}

pub struct Scene<'a> {
    memory: Bump,
    // Both of these are manually dropped to safe on memory as they won't
    // be needed when we are actually rendering:
    geometry_ids: ManuallyDrop<HashMap<String, &'a dyn Geometry>>,
    material_ids: ManuallyDrop<HashMap<String, &'a dyn Material>>,

    is_generated: bool, // whether or not we had generated a scene. If we did, don't accept any more stuff.
    models: Vec<Model<'a>>,
}

impl<'a> Scene<'a> {
    pub fn new() -> Self {
        Scene {
            memory: Bump::new(),
            geometry_ids: ManuallyDrop::new(HashMap::new()),
            material_ids: ManuallyDrop::new(HashMap::new()),
            is_generated: false,
            models: Vec::new(),
        }
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_geometry<T: Geometry>(&mut self, geometry: T, id: String) -> SimpleResult<()> {
        debug_assert!(!self.is_generated);

        // First check if we already have an ID that matches this one:
        if self.geometry_ids.contains_key(&id) {
            bail!("Geometry id: \"{}\" is not unique.", id);
        }

        // Allocate the memory. This is dirty and I feel dirty for doing it this way, but I can't figure out
        // a clean way of doing this without the lifetime getting in the way. Rust doesn't support self-referential
        // stuff as the lifetimes get in the way.
        let geometry =
            unsafe { transmute::<&mut T, &'a dyn Geometry>(self.memory.alloc(geometry)) };
        self.geometry_ids.insert(id, geometry);
        Ok(())
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_material<T: Material>(&mut self, material: T, id: String) -> SimpleResult<()> {
        debug_assert!(!self.is_generated);

        // First check if we already have an ID that matches this one:
        if self.material_ids.contains_key(&id) {
            bail!("Material id: \"{}\" is not unique.", id);
        }

        let material =
            unsafe { transmute::<&mut T, &'a dyn Material>(self.memory.alloc(material)) };
        self.material_ids.insert(id, material);
        Ok(())
    }

    pub fn add_model<T: Transform>(
        &mut self,
        geometry_id: &str,
        material_id: &str,
        transform: T,
    ) -> SimpleResult<()> {
        debug_assert!(!self.is_generated);

        let &geometry = match self.geometry_ids.get(geometry_id) {
            Some(g) => g,
            _ => bail!("Geometry with id: {} was not created before.", geometry_id),
        };
        let &material = match self.material_ids.get(material_id) {
            Some(m) => m,
            _ => bail!("Material with id: {} was not created before.", material_id),
        };

        // Because the transform could be either animated or not, we need to use dynamic dispatch to
        // determine what to do:
        let transform =
            unsafe { transmute::<&mut T, &'a dyn Transform>(self.memory.alloc(transform)) };
        self.models.push(Model {
            geometry,
            material,
            transform,
        });
        Ok(())
    }

    // This function essentially
    pub fn generate_scene(&mut self) {
        debug_assert!(!self.is_generated);

        // To save memory, we can clear the HashMaps as we
        // won't be accepting any more stuff anyways:
        unsafe {
            ManuallyDrop::drop(&mut self.geometry_ids);
            ManuallyDrop::drop(&mut self.material_ids);
        }

        

        // Need to build some sort of generalized thing for BVH construction
        self.is_generated = true;
    }
}
