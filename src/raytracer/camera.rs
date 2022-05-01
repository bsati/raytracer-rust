use crate::math;
use crate::raytracer::raytrace::Ray;

/// Representation of a camera in 3D space.
/// A camera is set at an origin / eye point with a certain viewing frustum constrained
/// by the lower left corner and horizontal / vertical bounds.
pub struct Camera {
    eye: math::Vector3,
    horizontal: math::Vector3,
    vertical: math::Vector3,
    lower_left: math::Vector3,
}

impl Camera {
    /// Constructs a new camera with the given parameters.
    /// Calculates the needed vectors to create the viewing frustum.
    ///
    /// # Arguments
    ///
    /// * `eye` the origin point / eye of the camera
    /// * `look_at` the center / look at point of the camera in the scene
    /// * `up` up vector of the camera (normal)
    /// * `fovy` used to calculate the spacial image height (different from the actual height of the image file)
    /// * `width` width of the image
    /// * `height` height of the image
    pub fn new(
        eye: math::Vector3,
        look_at: math::Vector3,
        up: math::Vector3,
        fovy: f64,
        width: usize,
        height: usize,
    ) -> Camera {
        let view_vec = look_at - eye;
        let distance = view_vec.len();
        let view = view_vec / distance;

        let image_height = 2.0 * distance * (0.5 * fovy / 180.0 * std::f64::consts::PI).tan();
        let image_width = width as f64 / height as f64 * image_height;

        let horizontal = view.cross(&up).normalized() * image_width / width as f64;
        let vertical = horizontal.cross(&view).normalized() * image_height / height as f64;

        let lower_left =
            look_at - horizontal * (0.5 * width as f64) - vertical * (0.5 * height as f64);

        Camera {
            eye,
            horizontal,
            vertical,
            lower_left,
        }
    }

    /// Spawns a new primary ray for a given pixel tracing from the camera.
    ///
    /// # Arguments
    ///
    /// * `x` coordinate of the pixel on the x-axis
    /// * `y` coordinate of the pxiel on the y-axis
    pub fn spawn_ray(&self, x: f64, y: f64) -> Ray {
        Ray::new(
            self.eye,
            self.lower_left + self.horizontal * x + self.vertical * y - self.eye,
        )
    }
}
