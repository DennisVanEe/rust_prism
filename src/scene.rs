use crate::memory::allocators::DynStackAlloc;
use crate::geometry::Geometry;
use crate::shading::material::Material;

use simple_error::{bail, SimpleResult};

use std::collections::HashMap;

// Because there is only one instance of a scene, all of the information that holds
// onto a scene is static:
const STACK_SIZE_GEOMETRY: usize = 5 * 1024 * 1024; // 5 MB
const STACK_SIZE_MATERIAL: usize = 5 * 1024 * 1024; // 5 MB

static mut geometry_memory: DynStackAlloc = DynStackAlloc::new(STACK_SIZE_GEOMETRY);
static mut material_memory: DynStackAlloc = DynStackAlloc::new(STACK_SIZE_MATERIAL);

static mut geometry_ids: HashMap<String, &'static dyn Geometry> = HashMap::new();
static mut material_ids: HashMap<String, &'static dyn Material> = HashMap::new();

pub fn add_geometry<T: Sized + Geometry>(geom: T, id: String) -> SimpleResult<()> {
    // First we check if the item exists:
    if geometry_ids.contains_key(&id) {
        bail!("Geometry with id: {} is not unique", id);
    }
    let geometry = geometry_memory.push::<'static, T>(geom);
    // Then we allocate memory for the geometry:
    geometry_ids.insert(id, geometry);
    Ok(())
}