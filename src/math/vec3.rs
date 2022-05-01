use serde::{Deserialize, Deserializer};
use std::ops::{self, Index, IndexMut};

/// Structure denoting points and vectors in 3D space.
/// It's coordinates `x`, `y`, `z` can be accessed by the corresponding fields.
#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    data: [f64; 3],
}

impl<'de> Deserialize<'de> for Vector3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: serde_yaml::Value = serde_yaml::Value::deserialize(deserializer).unwrap();
        let x = val.get(0).unwrap().as_f64().unwrap();
        let y = val.get(1).unwrap().as_f64().unwrap();
        let z = val.get(2).unwrap().as_f64().unwrap();
        Ok(Vector3::new(x, y, z))
    }
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
        Vector3 { data: [x, y, z] }
    }

    /// Returns the x coordinate of the vector
    pub fn x(&self) -> f64 {
        self.data[0]
    }

    /// Returns the y coordinate of the vector
    pub fn y(&self) -> f64 {
        self.data[1]
    }

    /// Returns the z coordinate of the vector
    pub fn z(&self) -> f64 {
        self.data[2]
    }

    /// Calculates the dot product of two vectors.
    ///
    /// # Arguments
    ///
    /// * `other` compute the dot product of `self` and `other`
    #[inline]
    pub fn dot(&self, other: &Vector3) -> f64 {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
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
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0],
        )
    }

    /// Returns a new normalized vector constructed from `self` if the length is `!= 0`.
    /// If the vector is of length `0` the function returns a clone of `self`.
    #[inline]
    pub fn normalized(&self) -> Vector3 {
        let norm = self.len();
        if norm != 0.0 {
            return Vector3::new(self[0] / norm, self[1] / norm, self[2] / norm);
        }
        self.clone()
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

    /// Reflects the vector for the normal `n`.
    ///
    /// # Arguments
    ///
    /// * `n` Normal-vector to use for reflection
    #[inline]
    pub fn reflect(&self, n: &Vector3) -> Vector3 {
        *self - *n * (2.0 * n.dot(self))
    }

    /// Returns a new Vector representing the minimum of both
    /// vectors coordinate-wise.
    ///
    /// # Arguments
    ///
    /// * `other` Other vector to build the minimum with
    #[inline]
    pub fn min(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            f64::min(self[0], other[0]),
            f64::min(self[1], other[1]),
            f64::min(self[2], other[2]),
        )
    }

    /// Returns a new Vector representing the maximum of both
    /// vectors coordinate-wise.
    ///
    /// # Arguments
    ///
    /// * `other` Other vector to build the maximum with
    #[inline]
    pub fn max(&self, other: &Vector3) -> Vector3 {
        Vector3::new(
            f64::max(self[0], other[0]),
            f64::max(self[1], other[1]),
            f64::max(self[2], other[2]),
        )
    }
}

impl Index<usize> for Vector3 {
    type Output = f64;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[idx]
    }
}

impl IndexMut<usize> for Vector3 {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.data[idx]
    }
}

/// Add implementation for Vector + Vector
impl ops::Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2])
    }
}

/// Sub implementation for Vector - Vector
impl ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2])
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
        Vector3::new(self[0] * rhs, self[1] * rhs, self[2] * rhs)
    }
}

/// Div implementation for Vector / scalar to create a scaled vector
impl ops::Div<f64> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f64) -> Vector3 {
        Vector3::new(self[0] / rhs, self[1] / rhs, self[2] / rhs)
    }
}

impl ops::Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Vector3 {
        Vector3::new(-self[0], -self[1], -self[2])
    }
}
