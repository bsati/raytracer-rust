use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r: r, g: g, b: b }
    }

    #[inline]
    pub fn to_output(&self) -> [i32; 3] {
        [
            (255.999 * self.r) as i32,
            (255.999 * self.g) as i32,
            (255.999 * self.b) as i32,
        ]
    }
}

#[derive(Debug)]
pub struct Image {
    width: usize,
    height: usize,
    pixel_colors: Vec<Color>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        Image {
            width: width,
            height: height,
            pixel_colors: Vec::with_capacity(width * height),
        }
    }

    #[inline]
    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn set_pixel_color(&mut self, x: usize, y: usize, color: Color) {
        let index = self.get_index(x, y);
        self.pixel_colors[index] = color;
    }

    pub fn write_image(&self, output_path: &std::path::Path) {
        let mut file = fs::File::create(output_path).unwrap();
        write!(file, "P3\n{} {}\n255\n", self.width, self.height);
        for i in 0..self.width {
            for j in 0..self.height {
                let output_color = self.pixel_colors[self.get_index(i, j)].to_output();
                writeln!(
                    file,
                    "{} {} {}",
                    output_color[0], output_color[1], output_color[2]
                );
            }
        }
    }
}
