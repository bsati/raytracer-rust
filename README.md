# raytracer-rust
 
Tiny raytracer written with the goal to re-iterate on the concepts of raytracing. This implementation is inspired by a uni project done in C++ with the intent to learn Rust and build my own raytracer.

## Scenes

Scenes are described in YAML files listing all the required information to render the image and the scene setup. Scene objects are currently only supported to be planes and spheres. Support for arbitrary meshes via `.obj` loading might be added in the future.
