# Scene File Format #

This document is needed to specify the scene file format used by PRISM.

Scene file formats are written in TOML.

## Geometry ##

Geometry refers to the mathematical description of a 3D object (like a collection of vertices representing a mesh or a point and radius representing a sphere).

### Basic Shapes ###

- **Sphere**: This represents a simple sphere.

```json
"sphere_geometry": {
    "id": "sphere_a",          // id of geometry used by scene model
    "rev_orientation": true,   // if true, normals point inward
    "radius": 1.0,             // radius of the sphere
}
```

### Mesh ###

This represents a geometric mesh. Right now, PRISM only supports .ply files, so if you set the file type to anything else it won't process it.

```json
"mesh_geometry": {
    "id": "sphere_a",             // id of geometry used by scene model
    "file_type": "ply",           // the file type
    "dir": "/models/sphere.ply",  // the location of the file representing it
}
```

## Scene Model ##

A Scene Model is a lightweight object that actually resides in the scene. It simply stores a pointer to both the geometry and the material that make it up. This way, you can reuse materials and geometries throughout the scene. A model also takes a transformation type. This is a geometry space to world space transformation (so where in the scene one is placing the model).

Geometries and materials have a unique name that you can use to identify which geometry and material belong to this model. An example is shown below:

```json
"scene_model": {
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
- **Matrix**: If you want to specify the matrix itself, you can do so. Be mindful that if it isn't affine and invertible, you might get problems. PRISM performs a check to make sure that this is the case and tells you if it's a problem. Matrices are represented in row-major order (an array of arrays, each of which is a row):
    ```json
    "transform": {
        "matrix": [1.0, 0.0, 0.0, 34.0,
                0.0, 1.0, 0.0, 29.0,
                0.0, 0.0, 1.0, 09.0,
                0.0, 0.0, 0.0, 01.0],
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
