use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufWriter, Write};
use std::ops::{AddAssign, Mul};

/// Struct representation of RGB-Colors
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
        Color { r: r, g: g, b: b }
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
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
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

/// Representation of an Image with a certain width and height and it's Pixel colors
#[derive(Debug)]
pub struct Image {
    width: usize,
    height: usize,
    pixel_colors: Vec<Color>,
}

impl Image {
    /// Creates a new Image with the given width and height and allocates
    /// the needed `pixel_colors` storage.
    ///
    /// # Arguments
    ///
    /// * `width` width of the image (row pixel count)
    /// * `height` height of the image (column pixel count)
    pub fn new(width: usize, height: usize) -> Image {
        Image {
            width: width,
            height: height,
            pixel_colors: vec![Color::new(0.0, 0.0, 0.0); width * height],
        }
    }

    /// Utility function to get the index for the specified `x` and `y` coordinates.
    /// Since the Image uses a linear Vector for storage and `(x, y)` denoting a grid point
    /// some calculation is needed to get the corresponding index.
    #[inline]
    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    fn get_vertically_flipped_index(&self, x: usize, y: usize) -> usize {
        (self.height - 1 - y) * self.width + x
    }

    /// Sets the Color of the Image pixel at coordinates `x` and `y` to the given `color`.
    pub fn set_pixel_color(&mut self, x: usize, y: usize, color: Color) {
        let idx = self.get_index(x, y);
        self.pixel_colors[idx] = color;
    }

    #[inline]
    fn to_u8_buf(&self) -> Box<[u8]> {
        // let result: Vec<u8> = self
        //     .pixel_colors
        //     .iter()
        //     .flat_map(|&c| c.to_output())
        //     .collect();
        let mut result = vec![0; 3 * self.width * self.height];
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let output_colors = self.pixel_colors[idx].to_output();

                let flipped_index = self.get_vertically_flipped_index(x, y);
                for c in 0..3 {
                    result[3 * flipped_index + c] = output_colors[c];
                }
            }
        }
        result.into_boxed_slice()
    }

    /// Writes the current Image data (Pixel colors) to a png file at the given `output_path`.
    ///
    /// # Arguments
    ///
    /// * `output_path` Path specifying the output file to write to (will be created if it doesn't exist and overriden if it exists)
    pub fn write_image(&self, output_path: &std::path::Path) {
        let parent_dir = output_path.parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();
        let file = fs::File::create(output_path).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&*self.to_u8_buf()).unwrap();
    }

    pub fn write_image_ppm(&self, output_path: &std::path::Path) {
        let parent_dir = output_path.parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();
        let mut file = fs::File::create(output_path).unwrap();
        write!(file, "P3\n{} {}\n255\n", self.width, self.height).unwrap();
        for j in 0..self.height {
            for i in 0..self.width {
                let output_color =
                    self.pixel_colors[self.get_vertically_flipped_index(i, j)].to_output();
                writeln!(
                    file,
                    "{} {} {}",
                    output_color[0], output_color[1], output_color[2]
                )
                .unwrap();
            }
        }
    }
}
