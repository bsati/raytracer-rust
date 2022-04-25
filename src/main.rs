mod math;
mod raytracer;

use std::path;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    scene_path: String,
    #[clap(short, long)]
    output_path: String,
    #[clap(short, long, default_value_t = 5)]
    depth: u8,
    #[clap(long, required = false, default_value = "uniform:2")]
    ssaa: raytracer::SuperSampling,
}

fn main() {
    // let args = Args::parse();
    // let scene_path = path::Path::new(&args.scene_path);
    // let output_path = path::Path::new(&args.output_path);
    // raytracer::compute_image(args.ssaa, args.depth, scene_path, output_path);
    let scene_path = path::Path::new("./scenes/spheres.yaml");
    let output_path = path::Path::new("./outputs/spheres.png");
    raytracer::compute_image(
        raytracer::SuperSampling::Jitter(4),
        5,
        scene_path,
        output_path,
    );
}
