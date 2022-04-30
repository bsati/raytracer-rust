# raytracer-rust
 
Tiny raytracer written with the goal to re-iterate on the concepts of raytracing. This implementation is inspired by a uni project done in C++ with the intent to learn Rust and build my own raytracer.

<p align="center">
  <img
    width="500"
    src="https://raw.githubusercontent.com/bsati/raytracer-rust/main/scenes/spheres/spheres.png"
    alt="Example spheres scene"
  />
</p>

## Scenes

Scenes are described in YAML files listing all the required information to render the image and the scene setup. Scene objects can be either be supplied by mathematical representations (spheres, planes) or abritrary meshes. Meshes can be loaded by supplying a `.obj` filepath in the scene configuration and having a corresponding `.mtl` Material library in the same path. Currently only pre-triangulated meshes are supported with the program panicking if the `.obj` file contains faces with more than three vertices.