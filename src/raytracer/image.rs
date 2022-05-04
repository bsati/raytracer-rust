use serde::{Deserialize, Deserializer};
use std::fs;
use std::io::BufWriter;
use std::ops::{Add, AddAssign, DivAssign, Mul};

use crate::math::Vector3;

/// Struct representation of RGB-Colors
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    /// Creates a new color struct with the given rgb values
    ///
    /// # Arguments
    ///
    /// * `r` red-channel value
    /// * `g` green-channel value
    /// * `b` blue-channel value
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r, g, b }
    }

    /// Clamps the color to a maximum of `1.0` to avoid over-saturation
    pub fn clamp(&mut self) {
        self.r = f64::min(self.r, 1.0);
        self.g = f64::min(self.g, 1.0);
        self.b = f64::min(self.b, 1.0);
    }

    /// Converts the current value to PPM compatible output values contained in an integer array.
    #[inline]
    pub fn to_output(&self) -> [u8; 3] {
        [
            (255.999 * self.r) as u8,
            (255.999 * self.g) as u8,
            (255.999 * self.b) as u8,
        ]
    }

    pub fn from_output(bytes: [u8; 3]) -> Color {
        Color::new(
            bytes[0] as f64 / 255.999,
            bytes[1] as f64 / 255.999,
            bytes[2] as f64 / 255.999,
        )
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: serde_yaml::Value = serde_yaml::Value::deserialize(deserializer).unwrap();
        let r = val.get(0).unwrap().as_f64().unwrap();
        let g = val.get(1).unwrap().as_f64().unwrap();
        let b = val.get(2).unwrap().as_f64().unwrap();
        Ok(Color::new(r, g, b))
    }
}

impl From<Vector3> for Color {
    fn from(vec: Vector3) -> Self {
        Color {
            r: vec.x(),
            g: vec.y(),
            b: vec.z(),
        }
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl DivAssign<f64> for Color {
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        Color::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Color {
        Color::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl PartialEq<Color> for Color {
    fn eq(&self, other: &Color) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }

    fn ne(&self, other: &Color) -> bool {
        !(self == other)
    }
}

#[inline]
fn to_u8_buf(pixel_colors: Vec<Vec<Color>>) -> Box<[u8]> {
    let result: Vec<u8> = pixel_colors
        .iter()
        .flatten()
        .flat_map(|&c| c.to_output())
        .collect();
    result.into_boxed_slice()
}

/// Writes the current Image data (Pixel colors) to a png file at the given `output_path`.
///
/// # Arguments
///
/// * `pixel_colors` Vector containing the colors in row -> column order (size: width * height)
/// * `width` width of the image
/// * `height` height of the image
/// * `output_path` Path specifying the output file to write to (will be created if it doesn't exist and overriden if it exists)
///
/// # Panics
///
/// If the file cannot be written to.
pub fn write_image(
    pixel_colors: Vec<Vec<Color>>,
    width: usize,
    height: usize,
    output_path: &std::path::Path,
) {
    let parent_dir = output_path.parent().unwrap();
    fs::create_dir_all(parent_dir).unwrap();
    let file = fs::File::create(output_path).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&*to_u8_buf(pixel_colors)).unwrap();
}

/// Reads and Image
pub fn read_image(file_path: &std::path::Path) -> (Vec<Color>, usize, usize) {
    let file = fs::File::open(file_path).unwrap();
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let len = info.buffer_size() / 3;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let idx = i * 3;
        result.push(Color::from_output([
            bytes[idx],
            bytes[idx + 1],
            bytes[idx + 2],
        ]));
    }
    (result, info.width as usize, info.height as usize)
}
