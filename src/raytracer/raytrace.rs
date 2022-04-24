use crate::math;
use crate::raytracer::camera;
use crate::raytracer::image;
use crate::raytracer::scene;
use serde_yaml;
use std::fs;
use std::path;

/// Basic structure representing a ray being cast into the scene.
/// A ray consists of an origin point `o` and a direction `d`. It's position can therefore
/// be calculated for any timestep `t` by `o + t * d`
pub struct Ray {
    pub origin: math::Vector3,
    pub direction: math::Vector3,
}

impl Ray {
    /// Creates a new ray with the given origin and direction vector
    ///
    /// # Arguments
    ///
    /// * `origin` Origin of the Ray (for primary rays this is the camera position / eye)
    /// * `direction` Direction of the ray to determine it's movement in space
    pub fn new(origin: math::Vector3, direction: math::Vector3) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
        }
    }

    /// Evaluates the ray at timestep `t`
    pub fn at_timestep(&self, t: f64) -> math::Vector3 {
        self.origin + self.direction * t
    }

    /// Traces the ray through the scene to calculate the resulting pixel color
    fn trace(&self) {}
}

/// Computes the image for a given scene config (loaded from `scene_path`) by raytracing and saves it to the specified `output_path`.
/// For more details on scene configs see [Scene](crate::raytracer::scene::Scene).
///
/// # Arguments
///
/// * `depth` determines the maximum ray bounce / tracing recursion depth
/// * `scene_path` Path to the scene file determining the needed properties for raytracing
/// * `output_path` Path of the output image file
pub fn compute_image(depth: u8, scene_path: &path::Path, output_path: &path::Path) {
    let scene_file = fs::File::open(scene_path).unwrap();
    let scene: scene::Scene = serde_yaml::from_reader(scene_file).unwrap();

    let mut img = image::Image::new(scene.image.width, scene.image.height);
    let camera = camera::Camera::new(
        scene.camera.eye,
        scene.camera.look_at,
        scene.camera.up,
        scene.camera.fovy,
        scene.image.width,
        scene.image.height,
    );
    for i in 0..scene.image.width {
        for j in 0..scene.image.height {
            let mut ray = camera.spawn_ray(i as f64, j as f64);
        }
    }
}
