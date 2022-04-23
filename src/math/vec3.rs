use std::ops;

#[derive(Debug, Clone)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    pub fn from_scalar(scalar: f64) -> Vector3 {
        Vector3 {
            x: scalar,
            y: scalar,
            z: scalar,
        }
    }

    #[inline]
    pub fn dot(&self, other: &Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn len(&self) -> f64 {
        f64::sqrt(self.sqr_len())
    }

    #[inline]
    pub fn sqr_len(&self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn normalized(&self) -> Vector3 {
        let norm = self.len();
        if norm != 0.0 {
            return Vector3::new(self.x / norm, self.y / norm, self.z / norm);
        }
        self.clone()
    }

    #[inline]
    pub fn normalize(&mut self) -> &mut Vector3 {
        let norm = self.len();
        if norm != 0.0 {
            self.x /= norm;
            self.y /= norm;
            self.z /= norm;
        }
        self
    }
}

impl ops::Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::Mul<Vector3> for Vector3 {
    type Output = f64;

    fn mul(self, rhs: Vector3) -> f64 {
        self.dot(&rhs)
    }
}

impl ops::Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f64) -> Vector3 {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::Div<f64> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f64) -> Vector3 {
        Vector3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}