use crate::{math::Vector3, raytracer::raytrace::Ray};

use super::{
    materials::Material,
    mesh::{Mesh, AABB},
    scene::{Plane, Sphere},
};

pub trait Intersectable {
    /// Checks if the ray intersects the object and returns the corresponding `IntersectionInfo` if it does
    /// or `None` otherwise
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo>;
}

/// Information about a ray-object intersection.
/// Contains the intersection point, normal, material of the intersected object and the `t` for which the intersection occurs.
#[derive(Clone, Copy, Debug)]
pub struct IntersectionInfo<'mat> {
    pub point: Vector3,
    pub normal: Vector3,
    pub material: &'mat Material,
    pub t: f64,
    pub u: Option<f64>,
    pub v: Option<f64>,
}

impl IntersectionInfo<'_> {
    pub fn new(point: Vector3, normal: Vector3, material: &Material, t: f64) -> IntersectionInfo {
        IntersectionInfo {
            point,
            normal,
            material,
            t,
            u: None,
            v: None,
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let dir = ray.direction;
        let oc = ray.origin - self.center;

        let a = dir.sqr_len();
        let b = 2.0 * dir.dot(&oc);
        let c = oc.sqr_len() - self.radius * self.radius;
        let mut d = b * b - 4.0 * a * c;

        // Check for intersection
        if d >= 0.0 {
            d = f64::sqrt(d);

            let t1 = (-b - d) / (2.0 * a);
            let t2 = (-b + d) / (2.0 * a);

            let mut intersection_t = f64::MAX;
            if t1 > 1e-5 && t1 < intersection_t {
                intersection_t = t1;
            }
            if t2 > 1e-5 && t2 < intersection_t {
                intersection_t = t2;
            }

            if intersection_t == f64::MAX {
                return None;
            }
            let intersection_point = ray.at_timestep(intersection_t);
            let intersection_normal = (intersection_point - self.center) / self.radius;

            return Some(IntersectionInfo::new(
                intersection_point,
                intersection_normal,
                &self.material,
                intersection_t,
            ));
        }
        None
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let dot_nd = self.normal.dot(&ray.direction);
        if f64::abs(dot_nd) < 1e-6 {
            return None;
        }

        let intersection_t = (self.center - ray.origin).dot(&self.normal) / dot_nd;

        if intersection_t < 1e-5 {
            return None;
        }

        let intersection_point = ray.at_timestep(intersection_t);
        let intersection_normal = self.normal;
        Some(IntersectionInfo::new(
            intersection_point,
            intersection_normal,
            &self.material,
            intersection_t,
        ))
    }
}

/// Calculates the determinant of a matrix represented by three column vectors.
///
/// Following the formula of:
/// | a b c |
/// | d e f |
/// | g h i |
/// det = (aei + bfg + cdh) - (ceg + bdi + afh)
///
/// which leads to
/// det = (v1.x * v2.y * v3.z + v2.x * v3.y * v1.z + v3.x * v1.y * v2.z) - (v3.x * v2.y * v1.z + v2.x * v1.y * v3.z + v1.x * v3.y * v2.z)
///
/// # Arguments
///
/// * `v1` Vector representing the left column
/// * `v2` Vector representing the middle column
/// * `v3` Vector representing the right column
#[inline]
fn calculate_determinant(v1: &Vector3, v2: &Vector3, v3: &Vector3) -> f64 {
    (v1[0] * v2[1] * v3[2] + v2[0] * v3[1] * v1[2] + v3[0] * v1[1] * v2[2])
        - (v3[0] * v2[1] * v1[2] + v2[0] * v1[1] * v3[2] + v1[0] * v3[1] * v2[2])
}

impl Intersectable for Mesh {
    /// Intersection testing of a mesh happens in two steps:
    /// - test the AABB of the mesh
    /// - test each triangle of the mesh and find the closest intersection (if any exist)
    ///
    /// Triangle intersection is implemented via barycentric coordinates.
    /// For a triangle constructed by the points `a`, `b`, `c` and a ray with origin `o` and direction `d`
    /// the equation `o + td = alpha * a + beta * b + (1 - alpha - beta) * c` has to be solved.
    /// This is done by using Cramers-Rule after rearranging the equation to:
    /// `[ d | (b-a) | (c-a) ] = (-t, alpha, beta)^T`
    /// The Matrix on the left hand side is represented as three column vectors.
    fn intersect(&self, ray: &Ray) -> Option<IntersectionInfo> {
        let mut result: Option<IntersectionInfo> = None;

        if let Some(bb) = &self.aabb {
            if !bb.intersect(ray) {
                return None;
            }
        }

        for triangle in &self.triangles {
            let pos_idx = triangle.vertex_idx;
            let a = self.vertex_positions[pos_idx[0]];
            let b = self.vertex_positions[pos_idx[1]];
            let c = self.vertex_positions[pos_idx[2]];
            let ab = b - a;
            let ac = c - a;

            let res = ray.origin - a;
            let det_m = calculate_determinant(&ray.direction, &ab, &ac);
            let det_m_t = calculate_determinant(&res, &ab, &ac);
            let det_m_a = calculate_determinant(&ray.direction, &res, &ac);
            let det_m_b = calculate_determinant(&ray.direction, &ab, &res);

            let a = det_m_a / det_m;
            let b = det_m_b / det_m;
            let t = -(det_m_t / det_m);

            if a < 0.0 || b < 0.0 || a + b > 1.0 || t < 0.0 {
                continue;
            }
            let normal = ab.cross(&ac).normalized();
            if result.is_none() || result.unwrap().t > t {
                let mut info = IntersectionInfo::new(
                    ray.at_timestep(t),
                    normal,
                    &self.materials[triangle.material_idx],
                    t,
                );
                if let Some(tuv_idx) = triangle.uv_idx {
                    let (u1, v1) = self.uvs[tuv_idx[1]];
                    let (u2, v2) = self.uvs[tuv_idx[2]];
                    let (u3, v3) = self.uvs[tuv_idx[0]];
                    info.u = Some(a * u1 + b * u2 + (1.0 - a - b) * u3);
                    info.v = Some(a * v1 + b * v2 + (1.0 - a - b) * v3);
                }
                result = Some(info);
            }
        }

        result
    }
}

impl AABB {
    /// Checks if the ray intersects the AABB and returns `true` if the ray intersects or false if it doesn't.
    /// The implementation is derived from Andrew Woo's: Fast Ray-Box Intersection implemented in C.
    fn intersect(&self, ray: &Ray) -> bool {
        const LEFT: u8 = 0;
        const RIGHT: u8 = 1;
        const MIDDLE: u8 = 2;
        const NONE: u8 = 3;

        let mut quadrant = [NONE; 3];
        let mut candidate_plane = [-1.0; 3];
        let mut inside = true;
        for i in 0..3 {
            if ray.origin[i] < self.min[i] {
                quadrant[i] = LEFT;
                candidate_plane[i] = self.min[i];
                inside = false;
            } else if ray.origin[i] > self.max[i] {
                quadrant[i] = RIGHT;
                candidate_plane[i] = self.max[i];
                inside = false;
            } else {
                quadrant[i] = MIDDLE;
            }
        }

        if inside {
            // coords = origin
            return true;
        }

        let mut max_t = [-1.0; 3];
        for i in 0..3 {
            if quadrant[i] != MIDDLE && ray.direction[i] != 0.0 {
                max_t[i] = (candidate_plane[i] - ray.origin[i]) / ray.direction[i];
            }
        }

        let mut which_plane = 0;
        for i in 0..3 {
            if max_t[which_plane] < max_t[i] {
                which_plane = i;
            }
        }

        let mut coords = [0.0; 3];
        for i in 0..3 {
            if which_plane != i {
                coords[i] = ray.origin[i] + max_t[which_plane] * ray.direction[i];
                if coords[i] < self.min[i] || coords[i] > self.max[i] {
                    return false;
                }
            }
            // else {
            //     coords[i] = candidate_plane[i];
            // }
        }
        return true;
    }
}

#[cfg(test)]
mod test {
    use crate::{
        math::Vector3,
        raytracer::{
            image::Color,
            raytrace::Ray,
            scene::{
                materials::{EmissiveMaterial, Material},
                mesh::{Mesh, Triangle, AABB},
                Plane, Sphere,
            },
        },
    };

    use super::Intersectable;

    #[test]
    fn test_aabb_intersection() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        let aabb = AABB::new(Vector3::new(0.0, 2.0, 0.0), Vector3::new(1.0, 2.0, 1.0));
        let aabb_neg = AABB::new(Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 1.0, 2.0));

        assert!(aabb.intersect(&ray));
        assert!(!aabb_neg.intersect(&ray));
    }

    #[test]
    fn test_aabb_intersection_inside() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let aabb = AABB::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        assert!(aabb.intersect(&ray));
    }

    #[test]
    fn test_mesh_intersection() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let mut mesh = Mesh::new();
        mesh.vertex_positions.push(Vector3::new(5.0, -1.0, -1.0));
        mesh.vertex_positions.push(Vector3::new(5.0, 1.0, 0.0));
        mesh.vertex_positions.push(Vector3::new(5.0, -1.0, 1.0));
        let mat = Material::Emissive(EmissiveMaterial::new(Color::new(1.0, 0.0, 0.0)));
        mesh.materials.push(mat.clone());
        let triangle = Triangle::new([0, 1, 2], 0);
        mesh.triangles.push(triangle);

        let intersection = mesh.intersect(&ray);
        assert!(intersection.is_some());
        let intersection = intersection.unwrap();
        assert_eq!(intersection.point, Vector3::new(5.0, 0.0, 0.0));
        assert_eq!(intersection.normal, Vector3::new(1.0, 0.0, 0.0));
        if let Material::Emissive(em) = mat {
            assert_eq!(em.color.r, 1.0);
        } else {
            assert!(false, "material not set correctly")
        }
        assert_eq!(intersection.t, 5.0);
    }

    #[test]
    fn test_mesh_intersection_negative() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let mut mesh = Mesh::new();
        mesh.vertex_positions.push(Vector3::new(5.0, 1.0, -1.0));
        mesh.vertex_positions.push(Vector3::new(5.0, 1.0, 0.0));
        mesh.vertex_positions.push(Vector3::new(5.0, 1.0, 1.0));
        let triangle = Triangle::new([0, 1, 2], 0);
        mesh.triangles.push(triangle);

        mesh.compute_aabb();

        let intersection = mesh.intersect(&ray);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_sphere_intersection() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let mat = Material::Emissive(EmissiveMaterial::new(Color::new(1.0, 0.0, 0.0)));
        let sphere = Sphere {
            center: Vector3::new(2.0, 0.0, 1.0),
            radius: 1.0,
            material: mat.clone(),
        };

        let intersection = sphere.intersect(&ray);
        assert!(intersection.is_some());
        let intersection = intersection.unwrap();
        assert_eq!(intersection.point, Vector3::new(2.0, 0.0, 0.0));
        assert_eq!(intersection.normal, Vector3::new(0.0, 0.0, -1.0));
        if let Material::Emissive(em) = mat {
            assert_eq!(em.color.r, 1.0);
        } else {
            assert!(false, "material not set correctly")
        }
        assert_eq!(intersection.t, 2.0);
    }

    #[test]
    fn test_sphere_intersection_negative() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let sphere = Sphere {
            center: Vector3::new(0.0, 1.0, 0.0),
            radius: 0.5,
            material: Material::Emissive(EmissiveMaterial::new(Color::new(1.0, 0.0, 0.0))),
        };

        let intersection = sphere.intersect(&ray);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_plane_intersection() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let mat = Material::Emissive(EmissiveMaterial::new(Color::new(1.0, 0.0, 0.0)));
        let plane = Plane {
            center: Vector3::new(2.0, 0.0, 0.0),
            normal: Vector3::new(-1.0, 0.0, 0.0),
            material: mat.clone(),
        };

        let intersection = plane.intersect(&ray);
        assert!(intersection.is_some());
        let intersection = intersection.unwrap();
        assert_eq!(intersection.point, Vector3::new(2.0, 0.0, 0.0));
        assert_eq!(intersection.normal, Vector3::new(-1.0, 0.0, 0.0));
        if let Material::Emissive(em) = mat {
            assert_eq!(em.color.r, 1.0);
        } else {
            assert!(false, "material not set correctly")
        }
        assert_eq!(intersection.t, 2.0);
    }

    #[test]
    fn test_plane_intersection_negative() {
        let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        let plane = Plane {
            center: Vector3::new(-1.0, 0.0, 0.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
            material: Material::Emissive(EmissiveMaterial::new(Color::new(1.0, 0.0, 0.0))),
        };

        let intersection = plane.intersect(&ray);
        assert!(intersection.is_none());
    }
}
