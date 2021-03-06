use rand::Rng;
use serde::{Deserialize, Deserializer};
use std::ops::{self, Index, IndexMut};

/// Structure denoting points and vectors in 3D space.
/// It's coordinates `x`, `y`, `z` can be accessed by the corresponding fields.
#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    data: [f64; 3],
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

    /// Creates a random vector with each coordinate in the given bounds.
    ///
    /// # Arguments
    ///
    /// * `min` minimum of the rng range
    /// * `max` maximum of the rng range (inclusive)
    pub fn random(min: f64, max: f64) -> Vector3 {
        let mut rng = rand::thread_rng();
        Vector3::new(
            rng.gen_range(min..=max),
            rng.gen_range(min..=max),
            rng.gen_range(min..=max),
        )
    }

    /// Creates a random vector in the unit sphere
    pub fn random_in_unit_sphere() -> Vector3 {
        loop {
            let v = Vector3::random(-1.0, 1.0);
            if v.sqr_len() < 1.0 {
                return v;
            }
        }
    }

    /// Creates a random unit vector
    pub fn random_unit_vector() -> Vector3 {
        Vector3::random_in_unit_sphere().normalized()
    }

    /// Checks if the vector is near zero meaning all it's coordinates are close to being 0.0
    pub fn near_zero(&self) -> bool {
        let epsilon = 1e-8;
        self[0].abs() < epsilon && self[1].abs() < epsilon && self[2].abs() < epsilon
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

    /// Reflects the vector for the normal `n`.
    ///
    /// # Arguments
    ///
    /// * `n` Normal-vector to use for reflection
    #[inline]
    pub fn reflect(&self, n: &Vector3) -> Vector3 {
        *self - *n * (2.0 * self.dot(n))
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

impl PartialEq<Vector3> for Vector3 {
    fn eq(&self, other: &Vector3) -> bool {
        self.data[0] == other.data[0]
            && self.data[1] == other.data[1]
            && self.data[2] == other.data[2]
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

#[cfg(test)]
mod test {
    use super::Vector3;

    #[test]
    fn test_dot() {
        let v1 = Vector3::new(0.0, 0.0, 0.0);
        let v2 = Vector3::new(1.0, 1.0, 1.0);
        let v3 = Vector3::new(2.0, 3.0, 4.0);

        let dot_v1_v2 = v1.dot(&v2);
        let dot_v1_v2_op = v1 * v2;
        let dot_v2_v3 = v2.dot(&v3);

        assert_eq!(dot_v1_v2, 0.0, "wrong dot product");
        assert_eq!(dot_v1_v2_op, 0.0, "operator not applying dot product");
        assert_eq!(dot_v2_v3, 9.0, "wrong dot product");
    }

    #[test]
    fn test_cross() {
        let v1 = Vector3::new(1.0, 0.0, 0.0);
        let v2 = Vector3::new(0.0, 1.0, 0.0);
        let v3 = Vector3::new(1.0, 0.0, 0.0);

        let cross_v1_v2 = v1.cross(&v2);
        let cross_v1_v3 = v1.cross(&v3);

        assert_eq!(
            cross_v1_v2,
            Vector3::new(0.0, 0.0, 1.0),
            "wrong cross product"
        );
        assert_eq!(
            cross_v1_v3,
            Vector3::new(0.0, 0.0, 0.0),
            "wrong cross product"
        );
    }

    #[test]
    fn test_normalized_length() {
        let v1 = Vector3::new(1.0, 2.0, 2.0);
        let v2 = Vector3::new(0.0, 0.0, 0.0);

        let v1_normalized = v1.normalized();
        let v2_normalized = v2.normalized();

        assert_eq!(
            v1_normalized,
            Vector3::new(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0),
            "wrong normalization of non-normalized vector"
        );

        assert_eq!(
            v2_normalized, v2,
            "wrong normalization of zero length vector"
        );
    }

    #[test]
    fn test_operators() {
        let v1 = Vector3::new(1.0, 1.0, 1.0);
        let v2 = Vector3::new(0.5, 0.5, 0.5);

        let sub = v1 - v2;
        let add = v1 + v2;
        let scale = v1 * 2.0;
        let scale_s = v1 / 2.0;
        let neg = -v1;

        assert_eq!(sub, Vector3::new(0.5, 0.5, 0.5), "wrong sub");
        assert_eq!(add, Vector3::new(1.5, 1.5, 1.5), "wrong add");
        assert_eq!(scale, Vector3::new(2.0, 2.0, 2.0), "wrong scale");
        assert_eq!(scale_s, v2, "wrong scale divide");
        assert_eq!(neg, Vector3::new(-1.0, -1.0, -1.0), "wrong negation");
    }

    #[test]
    fn test_near_zero() {
        let positive = Vector3::new(0.0000000000001, 0.0000000000001, 0.0000000000001);
        let negative = Vector3::new(0.00001, 0.00001, 0.00001);

        assert!(positive.near_zero());
        assert!(!negative.near_zero());
    }

    #[test]
    fn test_random_unit_vector() {
        let vec = Vector3::random_unit_vector();

        let len = vec.len();

        assert!(len >= 9e-6 && len <= 1.0 + 1e-6);
    }
}
