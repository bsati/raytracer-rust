use serde::{Deserialize, Serialize};
use std::ops;

/// Structure denoting points and vectors in 3D space.
/// It's coordinates `x`, `y`, `z` can be accessed by the corresponding fields.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Creates a new vector for the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` x value of the vector
    /// * `y` y value of the vector
    /// * `z` z value of the vector
    pub fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    /// Creates a new vector with every coordinate set to the given `scalar`.
    ///
    /// # Arguments
    ///
    /// * `scalar` value to set in all coordinates
    pub fn from_scalar(scalar: f64) -> Vector3 {
        Vector3 {
            x: scalar,
            y: scalar,
            z: scalar,
        }
    }

    /// Calculates the dot product of two vectors.
    ///
    /// # Arguments
    ///
    /// * `other` compute the dot product of `self` and `other`
    #[inline]
    pub fn dot(&self, other: &Vector3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculates the euclidian norm / length of the vector.
    #[inline]
    pub fn len(&self) -> f64 {
        f64::sqrt(self.sqr_len())
    }

    /// Calculates the squared euclidian norm / length of the vector.
    /// Used for comparing distances since this is faster than using [len](Self::len)
    #[inline]
    pub fn sqr_len(&self) -> f64 {
        self.dot(self)
    }

    /// Calculates the cross product of two vectors.
    ///
    /// # Arguments
    ///
    /// * `other` compute the cross product of `self` and `other`
    #[inline]
    pub fn cross(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Returns a new normalized vector constructed from `self` if the length is `!= 0`.
    /// If the vector is of length `0` the function returns a clone of `self`.
    #[inline]
    pub fn normalized(&self) -> Vector3 {
        let norm = self.len();
        if norm != 0.0 {
            return Vector3::new(self.x / norm, self.y / norm, self.z / norm);
        }
        self.clone()
    }

    /// Normalizes a vector in place.
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

    /// Constructs a new vector as a mirrored version of `self` along the normal `n`.
    ///
    /// # Arguments
    ///
    /// * `n` Normal acting as mirror axis
    #[inline]
    pub fn mirror(&self, n: &Vector3) -> Vector3 {
        return *n * (2.0 * self.dot(n)) - *self;
    }

    #[inline]
    pub fn reflect(&self, n: &Vector3) -> Vector3 {
        *self - *n * (2.0 * n.dot(self))
    }
}

/// Add implementation for Vector + Vector
impl ops::Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

/// Sub implementation for Vector - Vector
impl ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

/// Mul implementation for Vector * Vector calculating the dot product
impl ops::Mul<Vector3> for Vector3 {
    type Output = f64;

    fn mul(self, rhs: Vector3) -> f64 {
        self.dot(&rhs)
    }
}

/// Mul implementation for Vector * scalar to create a scaled vector
impl ops::Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f64) -> Vector3 {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// Div implementation for Vector / scalar to create a scaled vector
impl ops::Div<f64> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f64) -> Vector3 {
        Vector3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl ops::Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}
