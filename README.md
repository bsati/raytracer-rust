<p align="center">
  <img
    width="300"
    src="https://raw.githubusercontent.com/bsati/raytracer-rust/main/scenes/spheres/spheres.png"
    alt="Example spheres scene"
  />
</p>

# raytracer-rust
 
Tiny raytracer written with the goal to re-iterate on the concepts of raytracing. This implementation is built by somewhat following [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html) with the intent to learn Rust.

## Scenes

Scenes are described in YAML files listing all the required information to render the image and the scene setup. Scene objects can be either be supplied by mathematical representations (spheres, planes) or abritrary meshes. Meshes can be loaded by supplying a `.obj` filepath in the scene configuration. Only pre-triangulated meshes are supported with the program panicking if the `.obj` file contains faces with more than three vertices. Materials for meshes are only supported to be configured in the YAML file since the corresponding material library `.mtl` does not support different material types used in this project.