mod math;
mod raytracer;

use std::env;
use std::path;

fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 3 {
    //     panic!("less than 2 arguments supplied. need scene path and output path");
    // }
    // let scene_path = path::Path::new(&args[1]);
    // let output_path = path::Path::new(&args[2]);
    let scene_path = path::Path::new("./scenes/spheres.yaml");
    let output_path = path::Path::new("./outputs/spheres.ppm");
    raytracer::compute_image(scene_path, output_path);
}
