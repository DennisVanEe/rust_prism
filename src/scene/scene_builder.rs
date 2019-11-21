use super::{SceneObject, SceneObjectType};

use crate::geometry::Geometry;
use crate::shading::material::Material;
use crate::transform::Transform;

use bumpalo::Bump;
use simple_error::{bail, SimpleResult};

use std::collections::HashMap;
use std::mem::{transmute, ManuallyDrop};

// This is used to setup all of the scene information:
pub struct SceneBuilder<'a> {
    pub(super) allocator: Bump,
    // Both of these are manually dropped to safe on memory as they won't
    // be needed when we are actually rendering:
    pub(super) geometry_ids: ManuallyDrop<HashMap<String, &'a dyn Geometry>>,
    pub(super) material_ids: ManuallyDrop<HashMap<String, &'a dyn Material>>,
    pub(super) models: Vec<SceneObject<'a>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn new() -> Self {
        SceneBuilder {
            allocator: Bump::new(),
            geometry_ids: ManuallyDrop::new(HashMap::new()),
            material_ids: ManuallyDrop::new(HashMap::new()),
            models: Vec::new(),
        }
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_geometry<T: Geometry>(&mut self, geometry: T, id: String) -> SimpleResult<()> {
        // First check if we already have an ID that matches this one:
        if self.geometry_ids.contains_key(&id) {
            bail!("Geometry id: \"{}\" is not unique.", id);
        }

        // Allocate the memory. This is dirty and I feel dirty for doing it this way, but I can't figure out
        // a clean way of doing this without the lifetime getting in the way. Rust doesn't support self-referential
        // stuff as the lifetimes get in the way.
        let geometry =
            unsafe { transmute::<&mut T, &'a dyn Geometry>(self.allocator.alloc(geometry)) };
        self.geometry_ids.insert(id, geometry);
        Ok(())
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_material<T: Material>(&mut self, material: T, id: String) -> SimpleResult<()> {
        // First check if we already have an ID that matches this one:
        if self.material_ids.contains_key(&id) {
            bail!("Material id: \"{}\" is not unique.", id);
        }

        let material =
            unsafe { transmute::<&mut T, &'a dyn Material>(self.allocator.alloc(material)) };
        self.material_ids.insert(id, material);
        Ok(())
    }

    // A scene object associated with a material:
    pub fn add_material_object<T: Transform>(
        &mut self,
        geometry_id: &str,
        material_id: &str,
        transform: T,
    ) -> SimpleResult<()> {
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
        let geom_to_world =
            unsafe { transmute::<&mut T, &'a dyn Transform>(self.allocator.alloc(transform)) };
        self.models.push(SceneObject {
            geometry,
            obj_type: SceneObjectType::Material(material),
            geom_to_world,
        });
        Ok(())
    }
}
