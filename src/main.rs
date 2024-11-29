use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use rand::Rng;
use rayon::prelude::*;

fn main() {
    let (w, h) = (512, 512);
    let mut atomic_pixels: Vec<AtomicU32> = (0..w * h).map(|_| AtomicU32::new(0)).collect();
    (0..w*h*10).into_par_iter().for_each(|i| {
        let mut rng = rand::thread_rng();
        let (y, x): (f64, f64) = (rng.gen(), rng.gen());
        let (w, h) = (w as f64, h as f64);
        let i = ((y * h).floor() * w + (x * w).clamp(0., w - 1.).floor()) as usize;
        let (y, x) = ((y - 0.5) * 4., (x - 0.5) * 4.);

        let mut a: f64 = x;
        let mut b: f64 = y;
        for _ in 0..255 {
            // z^2 + c
            // (a, b) = (a*a - b*b, 2.*a*b);

            // z^i + c = e^(i ln z) + c

            let mut distance = (a*a + b*b).sqrt().ln();
            let mut angle = b.atan2(a);
            // (distance, angle) = (distance * -4., angle * -4.);
            (distance, angle) = (distance * 2., angle * -2.);
            (distance, angle) = (-angle, distance);
            let distance_back = distance.exp();
            let (a_base, b_base) = (angle.cos(), angle.sin());
            (a, b) = (distance_back * a_base, distance_back * b_base);


            (a, b) = (a + x, b + y);
        }

        let color = if (a*a + b*b) > 4. {
            (255, 255, 255)
        } else {
            (5, 5, 5)
        };

        atomic_pixels[i].store(color.0 + color.1 * 256 + color.2 * 256 * 256, std::sync::atomic::Ordering::Relaxed);
    });
    let pixels: Vec<[u8; 3]> = atomic_pixels.into_par_iter().map(|p| {
        let p: u32 = p.load(std::sync::atomic::Ordering::Relaxed);
        let r = (p % 256, (p / 256) % 256, (p / 256 / 256) % 256);
        [r.0 as u8, r.1 as u8, r.2 as u8]
    }).collect();
    let out_file = File::create(Path::new("assets/out.png")).unwrap();
    let out_writer = BufWriter::new(out_file);
    let mut encoder = png::Encoder::new(out_writer, w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let data = pixels.into_iter().flatten().collect::<Vec<_>>();
    writer.write_image_data(&data).unwrap();
}
