use super::{SceneObject, SceneObjectType};

use crate::geometry::Geometry;
use crate::shading::material::Material;
use crate::transform::Transform;

use simple_error::{bail, SimpleResult};

use std::collections::HashMap;
use std::mem::{transmute, ManuallyDrop};

/// A `SceneBuilder1 is used to create a scene by allocating memory for the different scene objects
/// and whatnot.
pub struct SceneBuilder<'a> {
    pub(super) geometry_ids: ManuallyDrop<HashMap<String, Box<dyn Geometry>>>,
    pub(super) material_ids: ManuallyDrop<HashMap<String, Box<dyn Material>>>,
    pub(super) models: Vec<SceneObject<'a>>,
}

impl<'a> SceneBuilder<'a> {
    pub fn new() -> Self {
        SceneBuilder {
            geometry_ids: ManuallyDrop::new(HashMap::new()),
            material_ids: ManuallyDrop::new(HashMap::new()),
            models: Vec::new(),
        }
    }

    // We want id to move ownership of the string. That should be more efficient.
    pub fn add_geometry(&mut self, geometry: T, id: String) -> SimpleResult<()> {
        // First check if we already have an ID that matches this one:
        if self.geometry_ids.contains_key(&id) {
            bail!("Geometry id: \"{}\" is not unique.", id);
        }

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
        let &geom = match self.geometry_ids.get(geometry_id) {
            Some(g) => g,
            _ => bail!("Geometry with id: {} was not created before.", geometry_id),
        };
        let &material = match self.material_ids.get(material_id) {
            Some(m) => m,
            _ => bail!("Material with id: {} was not created before.", material_id),
        };

        // Because the transform could be either animated or not, we need to use dynamic dispatch to
        // determine what to do:
        let geom_to_scene =
            unsafe { transmute::<&mut T, &'a dyn Transform>(self.allocator.alloc(transform)) };
        self.models.push(SceneObject {
            geom,
            obj_type: SceneObjectType::Material(material),
            geom_to_scene,
        });
        Ok(())
    }
}
