use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use rayon::prelude::*;

fn main() {
    let (w, h) = (512, 512);
    let pixels = (0..w * h).into_par_iter().map(|i| {
        let (y, x) = (i / h, i % h);
        let (y, x) = (y as f64 / h as f64, x as f64 / w as f64);
        let (y, x) = ((y - 0.5) * 4., (x - 0.5) * 4.);

        let mut a: f64 = x;
        let mut b: f64 = y;
        for _ in 0..30 {
            // z^2 + c
            // (a, b) = (a*a - b*b, 2.*a*b);

            // z^i + c = e^(i ln z) + c

            let mut distance = (a*a + b*b).sqrt().ln();
            let mut angle = b.atan2(a);
            // (distance, angle) = (distance * -4., angle * -4.);
            (distance, angle) = (distance * -4., angle * 4.);
            (distance, angle) = (-angle, distance);
            let distance_back = distance.exp();
            let (a_base, b_base) = (angle.cos(), angle.sin());
            (a, b) = (distance_back * a_base, distance_back * b_base);


            (a, b) = (a + x, b + y);
        }

        if (a*a + b*b) > 4. {
            return (255, 255, 255);
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
