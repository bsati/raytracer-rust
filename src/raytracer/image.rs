use std::fs;
use std::io::Write;

#[derive(Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
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
pub struct Image<const W: usize, const H: usize> {
    pixel_colors: [[Color; W]; H],
}

impl<const W: usize, const H: usize> Image<W, H> {
    pub fn new(pixel_colors: [[Color; W]; H]) -> Image<W, H> {
        Image {
            pixel_colors: pixel_colors,
        }
    }

    pub fn set_pixel_color(&mut self, x: usize, y: usize, color: Color) {
        self.pixel_colors[y][x] = color;
    }

    pub fn write_image(&self, output_path: &std::path::Path) {
        let mut file = fs::File::create(output_path).unwrap();
        write!(file, format!("P3\n{} {}\n255\n", W, H));
        for i in 0..W {
            for j in 0..H {
                let output_color = self.pixel_colors[j][i].to_output();
                writeln!(
                    file,
                    format!(
                        "{} {} {}",
                        output_color[0], output_color[1], output_color[2]
                    )
                );
            }
        }
        file.flush();
    }
}
