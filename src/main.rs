use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use rayon::prelude::*;

fn main() {
    let (w, h) = (512, 512);
    let pixels = (0..w * h).into_par_iter().map(|i| {
        let (y, x) = (i / h, i % h);
        let (y, x) = (y as f32 / h as f32, x as f32 / w as f32);
        let (y, x) = ((y - 0.5) * 4., (x - 0.5) * 4.);

        let mut a = 0.0;
        let mut b = 0.0;
        for _ in 0..100 {
            // (a+bi)(a+bi)
            // a^2 - b^2 + 2abi
            (a, b) = (a*a - b*b + x, 2.*a*b + y);
            if (a.abs() + b.abs()) > 2. {
                return (255, 255, 255);
            }
        }

        (5, 5, 5)
    });
    let out_file = File::create(Path::new("assets/out.png")).unwrap();
    let out_writer = BufWriter::new(out_file);
    let mut encoder = png::Encoder::new(out_writer, w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let data = pixels.map(|x| [x.0, x.1, x.2]).flatten().collect::<Vec<_>>();
    writer.write_image_data(&data).unwrap();
}
