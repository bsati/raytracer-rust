use crate::math::Vector3;
use crate::raytracer::anti_aliasing;
use crate::raytracer::camera;
use crate::raytracer::image;
use crate::raytracer::image::Color;
use crate::raytracer::scene;

use rand::Rng;
use rayon::prelude::*;
use serde_yaml;
use std::fs;
use std::path;

use super::scene::materials::Material;
use super::scene::materials::Scatter;

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
            origin,
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
    fn trace(&self, scene_config: &scene::SceneConfig, current_depth: u8, max_depth: u8) -> Color {
        if current_depth == max_depth {
            return Color::new(0.0, 0.0, 0.0);
        }

        let intersection = scene_config.scene.get_closest_interesection(self);
        if let Some(intersection_info) = &intersection {
            let scattered = intersection_info.material.scatter(self, intersection_info);
            return match scattered {
                Some((scattered_ray, albedo)) => match scattered_ray {
                    Some(scatter) => {
                        let scattered_color =
                            scatter.trace(scene_config, current_depth + 1, max_depth);

                        let mut prob = 0.1;

                        if let Material::Dieletrics(_) = intersection_info.material {
                            prob = 0.05;
                        }

                        let mut rng = rand::thread_rng();

                        let mut light_color = Color::new(0.0, 0.0, 0.0);
                        let lights_len = scene_config.scene.lights.len() as f64;
                        if lights_len > 0.0
                            && rng.gen::<f64>() > (1.0 - lights_len * prob)
                            && current_depth == (max_depth - 1)
                        {
                            for l in &scene_config.scene.lights {
                                let shadow_ray = Ray::new(
                                    intersection_info.point,
                                    l.sample_points[0] - intersection_info.point,
                                );
                                let target_color = shadow_ray.trace(scene_config, 0, 1);
                                light_color += albedo * target_color
                            }
                            light_color /= lights_len;
                        }
                        light_color + (albedo * scattered_color)
                    }
                    None => albedo,
                },
                None => Color::new(0.0, 0.0, 0.0),
            };
        }
        scene_config.image.background
    }
}

/// Computes the image for a given scene config (loaded from `scene_path`) by raytracing and saves it to the specified `output_path`.
/// For more details on scene configs see [Scene](crate::raytracer::scene::Scene).
///
/// # Arguments
///
/// * `ssaa` Algorithm to use for super sampling anti aliasing
/// * `depth` determines the maximum ray bounce / tracing recursion depth
/// * `scene_path` Path to the scene file determining the needed properties for raytracing
/// * `output_path` Path of the output image file
pub fn compute_image(
    ssaa: anti_aliasing::SuperSampling,
    depth: u8,
    scene_path: &path::Path,
    output_path: &path::Path,
) {
    let scene_file = fs::File::open(scene_path).unwrap();
    let mut scene_config: scene::SceneConfig = serde_yaml::from_reader(scene_file).unwrap();

    let camera = camera::Camera::new(
        scene_config.camera.eye,
        scene_config.camera.look_at,
        scene_config.camera.up,
        scene_config.camera.fovy,
        scene_config.image.width,
        scene_config.image.height,
    );
    scene_config.scene.precompute();
    let pixel_colors: Vec<Vec<Color>> = (0..scene_config.image.height)
        .into_par_iter()
        .rev()
        .map(|j: usize| {
            (0..scene_config.image.width)
                .into_par_iter()
                .map(|i: usize| {
                    let samples = ssaa.sample(i, j);
                    let mut pixel_color = image::Color::new(0.0, 0.0, 0.0);
                    let count = samples.len();
                    let samples_color = samples
                        .into_par_iter()
                        .map(|sample| {
                            let ray = camera.spawn_ray(sample.0, sample.1);
                            ray.trace(&scene_config, 0, depth)
                        })
                        .reduce(|| Color::new(0.0, 0.0, 0.0), |a, b| a + b);
                    pixel_color += samples_color;
                    pixel_color /= count as f64;
                    // Gamma adjustment
                    pixel_color.r = pixel_color.r.sqrt();
                    pixel_color.g = pixel_color.g.sqrt();
                    pixel_color.b = pixel_color.b.sqrt();
                    pixel_color.clamp();
                    pixel_color
                })
                .collect()
        })
        .collect();
    image::write_image(
        pixel_colors,
        scene_config.image.width,
        scene_config.image.height,
        output_path,
    );
}
