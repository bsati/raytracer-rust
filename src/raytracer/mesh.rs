use crate::math::Vector3;
use crate::raytracer::image::Color;
use crate::raytracer::scene::Material;
use serde::Deserialize;
use std::{
    collections::HashMap, fmt::Debug, fs::File, io::BufRead, io::BufReader, iter::Peekable,
    str::FromStr,
};

/// Loads an .obj file and returns a Vec containing all included meshes and their information
/// needed for raytracing.
///
/// # Arguments
///
/// * `file_path` Path to the .obj file. If a material library is used, the parent will be used to search for the .mtl file
pub fn load_obj(file_path: &std::path::Path) -> Vec<Mesh> {
    let obj_file = File::open(file_path).unwrap();
    let reader = BufReader::new(obj_file);

    let mut result = Vec::new();

    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut index_helper = IndexHelper::new();
    let mut active_object_index = usize::MAX;

    let mut material_index = usize::MAX;

    for line in reader.lines() {
        let l = line.unwrap();
        let mut values = l.split_whitespace().peekable();
        let header = values.next();
        match header {
            Some("mtllib") => {
                load_material_lib(
                    &file_path.parent().unwrap().join(values.next().unwrap()),
                    &mut materials,
                );
            }
            Some("o") => {
                let new_obj = Mesh::new();
                material_index = usize::MAX;
                result.push(new_obj);
                if active_object_index == usize::MAX {
                    active_object_index = 0;
                } else {
                    index_helper.add_object(&result[active_object_index]);
                    active_object_index += 1;
                }
            }
            //Some("s") => {} ignored for now
            Some("v") => {
                result[active_object_index]
                    .vertex_positions
                    .push(parse_vec(&mut values));
            }
            Some("vn") => {
                result[active_object_index]
                    .normals
                    .push(parse_vec(&mut values));
            }
            Some("vt") => {
                let u = parse_next(&mut values);
                let v = parse_next(&mut values);
                result[active_object_index].uvs.push((u, v));
            }
            Some("f") => {
                let format = FaceFormat::determine_format(&mut values);
                result[active_object_index]
                    .triangles
                    .push(format.get_triangle(&mut values, &index_helper, material_index));
            }
            Some("usemtl") => {
                let mat = materials.get(values.next().unwrap()).unwrap();
                result[active_object_index].materials.push(mat.clone());
                material_index = result[active_object_index].materials.len() - 1;
            }
            _ => continue,
        }
    }

    result
}

/// Enum representing the different formats of face-descriptions
/// in .obj files
enum FaceFormat {
    /// Face represented by only vertex positions
    /// e.g. f 1 2 3
    VPos,
    /// Face represented by vertex positions and uv coordinates
    /// e.g. f 1/1 2/2 3/3
    VPosUv,
    /// Face represented by vertex positions and normals
    /// e.g. f 1//1 2//2 3//3
    VPosN,
    /// Face represented by vertex positions, uv coordinates and normals
    /// e.g. f 1/1/1 2/2/2 3/3/3
    VPosUvN,
}

impl FaceFormat {
    /// Determines the face format by peeking at the first element and checking for following rules
    ///
    /// - no slash in the element -> only positions
    /// - double slash in the first element -> positions + normals
    /// - slash in the first element, first occurence from left = first occurence from right (no second slash) -> positions + uv coordinates
    /// - else positions + uv coordinates + normals
    ///
    /// # Arguments
    ///
    /// * `split` Peekable iterator over the split of arguments
    fn determine_format<'a, I: Iterator<Item = &'a str>>(split: &mut Peekable<I>) -> FaceFormat {
        let first = split.peek().unwrap();
        if first.contains("//") {
            return FaceFormat::VPosN;
        }
        let ind = first.find("/");
        if let Some(i) = ind {
            let ind_r = first.rfind("/").unwrap();
            if i == ind_r {
                return FaceFormat::VPosUv;
            }
            return FaceFormat::VPosUvN;
        }

        FaceFormat::VPos
    }

    /// Creates a Triangle struct by parsing the arguments.
    ///
    /// # Arguments
    ///
    /// * `split` Iterator over the string arguments to be parsed
    /// * `idx_helper` Index helper to reduce global file indices to local mesh indices
    /// * `mat_idx` Index of the currently "active" material to use for the triangle
    fn get_triangle<'a, I: Iterator<Item = &'a str>>(
        &self,
        split: &mut I,
        idx_helper: &IndexHelper,
        mat_idx: usize,
    ) -> Triangle {
        match self {
            FaceFormat::VPos => {
                let i1 = idx_helper.get_vertex_index(parse_next(split));
                let i2 = idx_helper.get_vertex_index(parse_next(split));
                let i3 = idx_helper.get_vertex_index(parse_next(split));
                Triangle::new([i1, i2, i3], mat_idx)
            }
            FaceFormat::VPosUv => {
                let (mut v_idx, mut uv_idx) = get_tuple_index(split, "/");
                for i in 0..3 {
                    v_idx[i] = idx_helper.get_vertex_index(v_idx[i]);
                    uv_idx[i] = idx_helper.get_uv_index(uv_idx[i]);
                }
                let mut t = Triangle::new(v_idx, mat_idx);
                t.uv_idx = Some(uv_idx);

                t
            }
            FaceFormat::VPosN => {
                let (v_idx, n_idx) = get_tuple_index(split, "//");
                let mut t = Triangle::new(v_idx, mat_idx);
                t.normal_idx = Some(n_idx);

                t
            }
            FaceFormat::VPosUvN => {
                let mut v_idx: [usize; 3] = [0; 3];
                let mut uv_idx: [usize; 3] = [0; 3];
                let mut n_idx: [usize; 3] = [0; 3];
                for i in 0..3 {
                    let next = split.next().unwrap().to_string();
                    let mut indices = next.split("/");
                    v_idx[i] = idx_helper.get_vertex_index(parse_next(&mut indices));
                    uv_idx[i] = idx_helper.get_uv_index(parse_next(&mut indices));
                    n_idx[i] = idx_helper.get_normals_index(parse_next(&mut indices));
                }
                let mut t = Triangle::new(v_idx, mat_idx);
                t.normal_idx = Some(n_idx);
                t.uv_idx = Some(uv_idx);

                t
            }
        }
    }
}

/// Helper function to get a tuple index of a FaceFormat
///
/// # Arguments
///
/// * `split` Iterator over the arguments to parse
/// * `split_pat` Pattern to to use for splitting indices
fn get_tuple_index<'a, I: Iterator<Item = &'a str>>(
    split: &mut I,
    split_pat: &str,
) -> ([usize; 3], [usize; 3]) {
    let mut idx1_arr: [usize; 3] = [0, 0, 0];
    let mut idx2_arr: [usize; 3] = [0, 0, 0];
    for (i, value) in split.enumerate() {
        let parse_string = value.to_string();
        let split_idx = parse_string.to_string().find(split_pat).unwrap();
        let idx1 = parse_string
            .get(..split_idx)
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let idx2 = parse_string
            .get(split_idx + 1..)
            .unwrap()
            .parse::<usize>()
            .unwrap();
        idx1_arr[i] = idx1;
        idx2_arr[i] = idx2;
    }
    (idx1_arr, idx2_arr)
}

/// Loading function for a material library which adds all loaded materials by name to the given HashMap.
///
/// # Arguments
///
/// * `file_path` path of the material library (.mtl)
/// * `material_map` mutable map to store the materials in
fn load_material_lib(file_path: &std::path::Path, material_map: &mut HashMap<String, Material>) {
    let mtl_file = File::open(file_path).unwrap();
    let reader = BufReader::new(mtl_file);
    let mut active_material = String::new();
    for line in reader.lines() {
        let l = line.unwrap();
        let mut values = l.split_whitespace();
        let header = values.next();
        match header {
            Some("newmtl") => {
                active_material = values.next().unwrap().to_string();
                material_map.insert(active_material.clone(), Material::default());
            }
            Some("Ka") => {
                material_map
                    .get_mut(&active_material)
                    .unwrap()
                    .ambient_color = Color::from(parse_vec(&mut values));
            }
            Some("Kd") => {
                material_map
                    .get_mut(&active_material)
                    .unwrap()
                    .diffuse_color = Color::from(parse_vec(&mut values));
            }
            Some("Ks") => {
                material_map
                    .get_mut(&active_material)
                    .unwrap()
                    .specular_color = Color::from(parse_vec(&mut values));
            }
            // Some("Ni") => {
            //     material_map
            //         .get_mut(&active_material)
            //         .unwrap()
            //         .optical_density = parse_next(&mut values);
            // } ignored for now
            Some("Ns") => {
                material_map.get_mut(&active_material).unwrap().shininess = parse_next(&mut values);
            }
            // Some("d") => {
            //     material_map.get_mut(&active_material).unwrap().dissolve = parse_next(&mut values);
            // } ignored for now
            _ => continue,
        }
    }
}

/// Utility function to parse a Vector3 from the given Iterator
#[inline]
fn parse_vec<'a, I: Iterator<Item = &'a str>>(split: &mut I) -> Vector3 {
    let x = parse_next(split);
    let y = parse_next(split);
    let z = parse_next(split);
    Vector3::new(x, y, z)
}

/// Utility function to parse the next value of the iterator to a given type
#[inline]
fn parse_next<'a, T: FromStr, I: Iterator<Item = &'a str>>(split: &mut I) -> T
where
    <T as FromStr>::Err: Debug,
{
    split.next().unwrap().parse::<T>().unwrap()
}

/// Struct containing global counter information to use for
/// converting global finde indices to local mesh indices.
struct IndexHelper {
    vertex_count: usize,
    normals_count: usize,
    uv_count: usize,
}

impl IndexHelper {
    /// Creates a new IndexHelper with all counts set to 0
    fn new() -> IndexHelper {
        IndexHelper {
            vertex_count: 0,
            normals_count: 0,
            uv_count: 0,
        }
    }

    /// Increases internal counters by index amounts used in the object
    ///
    /// # Arguments
    ///
    /// * `obj` Mesh with filled vertex positions, uvs and normals
    fn add_object(&mut self, obj: &Mesh) {
        self.vertex_count += obj.vertex_positions.len();
        self.uv_count += obj.uvs.len();
        self.normals_count += obj.normals.len();
    }

    /// Returns the local index of a vertex
    ///
    /// # Arguments
    ///
    /// * `file_idx` Global file index of the vertex
    fn get_vertex_index(&self, file_idx: usize) -> usize {
        file_idx - self.vertex_count - 1
    }

    /// Returns the local index of a normal vector
    ///
    /// # Arguments
    ///
    /// * `file_idx` Global file index of the normal vector
    fn get_normals_index(&self, file_idx: usize) -> usize {
        file_idx - self.normals_count - 1
    }

    /// Returns the local index of a uv coordinates tuple
    ///
    /// # Arguments
    ///
    /// * `file_idx` Global file index of the tuple
    fn get_uv_index(&self, file_idx: usize) -> usize {
        file_idx - self.uv_count - 1
    }
}

/// Mesh represents a loaded mesh from within an .obj file.
/// The only supported face type is a triangle. Faces with more than three vertices will result in a panic.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub vertex_positions: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub uvs: Vec<(f64, f64)>,
    pub aabb: Option<AABB>,
}

impl Mesh {
    /// Creates a new Mesh
    fn new() -> Mesh {
        Mesh {
            triangles: Vec::new(),
            materials: Vec::new(),
            vertex_positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            aabb: None,
        }
    }

    /// Computes the AABB of the mesh and stores it in itsself
    pub fn compute_aabb(&mut self) {
        let mut bb_min = Vector3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut bb_max = Vector3::new(f64::MIN, f64::MIN, f64::MIN);
        for vp in &self.vertex_positions {
            bb_min = bb_min.min(vp);
            bb_max = bb_max.max(vp);
        }
        self.aabb = Some(AABB::new(bb_min, bb_max))
    }
}

/// Representation of an axis-aligned bounding box
#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Vector3,
    pub max: Vector3,
}

impl AABB {
    /// Creates a new AABB with the given min and max points
    ///
    /// # Arguments
    ///
    /// * `min` minimum bounding point of the AABB
    /// * `max` maximum bounding point of the AABB
    fn new(min: Vector3, max: Vector3) -> AABB {
        AABB { min, max }
    }
}

/// Triangle acts as an index struct representing a single triangle of a mesh.
/// The contained data is only used to index the mesh's lists.
#[derive(Clone, Debug, Deserialize)]
pub struct Triangle {
    pub vertex_idx: [usize; 3],
    pub normal_idx: Option<[usize; 3]>,
    pub uv_idx: Option<[usize; 3]>,
    pub material_idx: usize,
}

impl Triangle {
    /// Creates a new Triangle with given vertex indices and material index
    ///
    /// # Arguments
    ///
    /// * `vertex_idx` Array containing the three vertex indices in the vertex position list of the mesh
    /// * `material_idx` Index of the material in the mesh's material list
    fn new(vertex_idx: [usize; 3], material_idx: usize) -> Triangle {
        Triangle {
            vertex_idx,
            normal_idx: None,
            uv_idx: None,
            material_idx,
        }
    }
}
