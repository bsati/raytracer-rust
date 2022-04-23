use crate::math;

pub struct Ray {
    origin: math::Vector3,
    direction: math::Vector3,
}

impl Ray {
    fn new(origin: math::Vector3, direction: math::Vector3) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
        }
    }

    fn at_timestep(&self, t: f32) -> math::Vector3 {
        self.origin + self.direction * t
    }
}
