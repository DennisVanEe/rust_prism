# Scene File Format #

This document is needed to specify the scene file format used by PRISM.

Scene file formats are written in TOML.

## Mesh and Instancing ##

Mesh are a collection of attributes. Each attribute is seperated into a PLY file. Each attribute has a name unique to that specific mesh. In order to place the mesh in the world, one needs a mesh instance.

```json
"mesh": {
    "name": "collection_a", // Name of the mesh. Not referenced by materials, only by mesh instances
    "attributes": [
        {
            "name": "a",
            "path": "/path/to/file/a.ply",
            "transform": {
                // See section on transformations
            },
        },
        {
            "name": "b",
            "path": "/path/to/file/b.ply",
            "transform": {
                // See section on transformations
            },
        },
        //...
    ],
    // Transforms all of the values:
    "transform": {
        // See section on transformations
    },
}
```

```json
"mesh_instance": {
    "mesh": "collection_a",
    
}
```

Every geometry in a group must be unique. If you need copies of a mesh then you need to instance them at the `"master_group"` level (again, support for multi-level instancing is planned).

Below is an example of a `"master_group"`:

```json
"master_group": {
    "members": [
        {
            "geometry_id": "mesh_a",   // the geometry referenced by its id
            "transform": //...         // see the section on transformations
        },      
        {
            "sub_group_id": "group_b",   // the geometry referenced by its id
            "instance_id": "starts0",    // id of the sub_group instance
            "transform": //...           // see the section on transformations
        },
        {
            "sub_group_id": "group_b",   // the geometry referenced by its id
            "instance_id": "starts2",    // id of the sub_group instance
            "transform": //...           // see the section on transformations
        }
        //...
    ]
}
```
If one is instancing a `"sub_group"` then one must remember to include an instance_id. This is used to tie materials to geometries, allowing for different materials for different instances of the same geometry (see the material section for more details).

## Materials ##

A material is tied to a single geometry. If you need a geometry with partitioned materials then make a complex material or break the geometry down.

TODO: talk about materials

### Material Instance ###

A `"material_instance"` binds a material to a specific geometry.

```json
"material_instance": {
    "material_id": "lambertian_red", // The material being bound
    "instance_id": "red_cars",       // The id of this particular instance
    "geometries": [                  // A list of geometries to bind this material to
        {
            "geometry_id": "car_mesh", // this applies to ALL car_mesh geometries (instanced or not)
        },
        {
            "geometry_id": "bus_mesh", // this only applies to the bus_mesh in the master_group
            "instance_id": "",           
        },
        {
            "geometry_id": "",         // this only applies to ALL meshes in the "vans" sub_group
            "instance_id": "vans", 
        }
        {
            "geometry_id": "monster",  // this only applies to the master geoemtry in the trucks instance
            "instance_id": "trucks", 
        },
    ]
}
```

## Scene Objects ##

There are two types of scene objects, geometric object and group object. A geometric object is used for 

```json
"scene_object": {
    "geometry": "sphere_mesh", // name of geometry created before
    "material": "blue_matte",  // name of material created before
    "transform": {             // see transform section for more details
        "type": "translate",
        "trans": [34.3, 89.1, 90.8],
    }
}
```

## Transformations ##

Transformations are an important part of any scene. They describe how objects are positioned in the world and how they move (if at all).

Every transform has a type specified with it. Let's go over the different types that are currently available:

- **Identity**: If you want to specify a "noop" transformation in a place where one is expected, then simply place an identity transformation. Applying this will have no effect.
    ```json
    "transform": {
        "identity": true
    }
    ```

- **Translation**: Just a translation by a specified vector:
    ```json
    "transform": {
        "translation": [34.3, 89.1, 90.8],
    }
    ```
- **Rotation**: Rotates by degrees (in degrees, obviously) around the specified axis:
    ```json
    "transform": {
        "degrees": 275.0,
        "axis": [1, 0, 08],
    }
    ```
- **Scale**: Scales by the specified vector:
    ```json
    "transform": {
        "scale": [34.3, 89.1, 90.8],
    }
    ```
- **Matrix**: If you want to specify the matrix itself, you can do so as a 3x4 matrix. Matrices are represented in row-major order (an array of arrays, each of which is a row):
    ```json
    "transform": {
        "matrix": [1.0, 0.0, 0.0, 34.0,
                0.0, 1.0, 0.0, 29.0,
                0.0, 0.0, 1.0, 09.0],
                //0.0, 0.0, 0.0, 01.0], <- last row is implied
    }
    ```
- **Composite**: A single transformation defined as a number of transformations. This is essentially represented as an array of transformations. The order of the transformations defines the order in which they are applied (not necessarily the order in which the transformation's matrix representation is multiplied). So, the bottom example would first scale the object, then translate it. It is also important to note that you can't have a composite of animated transforms:
    ```json
    "transform": {
        "composite": [
            {
                "type": "scale",
                "vec": [3, 3, 3],
            },
            {
                "type": "translate",
                "vec": [34.3, 89.1, 90.8],
            },
        ],
    }
    ```
- **Animated**: These are transforms that animate from a given start transform to a given end transform. These transformations can be anything except for another animated transform:
    ```json
    "transform": {
        "start_transform": {
            "type": "scale",
            "vec": [3, 3, 3],
        },
        "end_transform": {
                "type": "translate",
                "vec": [34.3, 89.1, 90.8],
        },
        "start_time": 0.0,
        "end_time": 1.0,
    }
    ```
