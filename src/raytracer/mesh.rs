use crate::math::Vector3;
use crate::raytracer::image::Color;
use crate::raytracer::scene::Material;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::BufRead,
    io::BufReader,
    iter::Peekable,
    str::{FromStr, SplitWhitespace},
};

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

enum FaceFormat {
    VPos,
    VPosUv,
    VPosN,
    VPosUvN,
}

impl FaceFormat {
    fn determine_format(split: &mut Peekable<SplitWhitespace>) -> FaceFormat {
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

    fn get_triangle(
        &self,
        split: &mut Peekable<SplitWhitespace>,
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
                let (mut v_idx, mut uv_idx) = get_tuple_index(split);
                for i in 0..3 {
                    v_idx[i] = idx_helper.get_vertex_index(v_idx[i]);
                    uv_idx[i] = idx_helper.get_uv_index(uv_idx[i]);
                }
                let mut t = Triangle::new(v_idx, mat_idx);
                t.uv_idx = Some(uv_idx);

                t
            }
            FaceFormat::VPosN => {
                let (v_idx, n_idx) = get_tuple_index(split);
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

fn get_tuple_index(split: &mut Peekable<SplitWhitespace>) -> ([usize; 3], [usize; 3]) {
    let mut idx1_arr: [usize; 3] = [0, 0, 0];
    let mut idx2_arr: [usize; 3] = [0, 0, 0];
    for (i, value) in split.enumerate() {
        let parse_string = value.to_string();
        let split_idx = parse_string.to_string().find("/").unwrap();
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

#[inline]
fn parse_vec(split: &mut dyn Iterator<Item = &str>) -> Vector3 {
    let x = parse_next(split);
    let y = parse_next(split);
    let z = parse_next(split);
    Vector3::new(x, y, z)
}

#[inline]
fn parse_next<T: FromStr>(split: &mut dyn Iterator<Item = &str>) -> T
where
    <T as FromStr>::Err: Debug,
{
    split.next().unwrap().parse::<T>().unwrap()
}

struct IndexHelper {
    vertex_count: usize,
    normals_count: usize,
    uv_count: usize,
}

impl IndexHelper {
    fn new() -> IndexHelper {
        IndexHelper {
            vertex_count: 0,
            normals_count: 0,
            uv_count: 0,
        }
    }

    fn add_object(&mut self, obj: &Mesh) {
        self.vertex_count += obj.vertex_positions.len();
        self.uv_count += obj.uvs.len();
        self.normals_count += obj.normals.len();
    }

    fn get_vertex_index(&self, file_idx: usize) -> usize {
        file_idx - self.vertex_count - 1
    }

    fn get_normals_index(&self, file_idx: usize) -> usize {
        file_idx - self.normals_count - 1
    }

    fn get_uv_index(&self, file_idx: usize) -> usize {
        file_idx - self.uv_count - 1
    }
}

/// Mesh describes a loaded mesh from withing the .obj file.
/// The only supported face type is a triangle. Faces with more than three vertices will result in a panic.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
    pub vertex_positions: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub uvs: Vec<(f64, f64)>,
}

impl Mesh {
    fn new() -> Mesh {
        Mesh {
            triangles: Vec::new(),
            materials: Vec::new(),
            vertex_positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Triangle {
    pub vertex_idx: [usize; 3],
    pub normal_idx: Option<[usize; 3]>,
    pub uv_idx: Option<[usize; 3]>,
    pub material_idx: usize,
}

impl Triangle {
    fn new(vertex_idx: [usize; 3], material_idx: usize) -> Triangle {
        Triangle {
            vertex_idx,
            normal_idx: None,
            uv_idx: None,
            material_idx,
        }
    }
}
