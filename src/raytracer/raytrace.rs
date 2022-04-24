use crate::math::Vector3;
use crate::raytracer::camera;
use crate::raytracer::image;
use crate::raytracer::image::Color;
use crate::raytracer::scene;
use serde_yaml;
use std::fs;
use std::path;

/// Basic structure representing a ray being cast into the scene.
/// A ray consists of an origin point `o` and a direction `d`. It's position can therefore
/// be calculated for any timestep `t` by `o + t * d`
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    /// Creates a new ray with the given origin and direction vector
    ///
    /// # Arguments
    ///
    /// * `origin` Origin of the Ray (for primary rays this is the camera position / eye)
    /// * `direction` Direction of the ray to determine it's movement in space
    pub fn new(origin: Vector3, direction: Vector3) -> Ray {
        Ray {
            origin: origin,
            direction: direction.normalized(),
        }
    }

    /// Evaluates the ray at timestep `t`
    pub fn at_timestep(&self, t: f64) -> Vector3 {
        self.origin + self.direction * t
    }

    /// Traces the ray through the scene to calculate the resulting pixel color
    ///
    /// # Arguments
    ///
    /// * `scene_config` Configuration of the scene
    /// * `depth` if the material of the object is mirroring, depth defines the recursion depth for which to spawn
    ///           secondary rays
    fn trace(&self, scene_config: &scene::SceneConfig, depth: u8) -> Color {
        let intersection = scene_config.scene.get_closest_interesection(self);
        if let Some(intersection_info) = intersection {
            let color = scene_config.scene.compute_phong_lighting(
                &intersection_info.point,
                &intersection_info.normal,
                &-self.direction,
                &intersection_info.material,
            );

            return color;
        }
        scene_config.image.background
    }
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
    let scene_config: scene::SceneConfig = serde_yaml::from_reader(scene_file).unwrap();

    let mut img = image::Image::new(scene_config.image.width, scene_config.image.height);
    let camera = camera::Camera::new(
        scene_config.camera.eye,
        scene_config.camera.look_at,
        scene_config.camera.up,
        scene_config.camera.fovy,
        scene_config.image.width,
        scene_config.image.height,
    );
    for i in 0..scene_config.image.width {
        for j in 0..scene_config.image.height {
            let ray = camera.spawn_ray(i as f64, j as f64);
            let mut pixel_color = ray.trace(&scene_config, depth);
            pixel_color.clamp();
            img.set_pixel_color(i, j, pixel_color);
        }
    }
    img.write_image(output_path);
}
