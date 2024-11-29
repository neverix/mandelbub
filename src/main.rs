use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use rand::Rng;
use rayon::prelude::*;


const N_ITERATIONS: usize = 50;
const SCALING_FACTOR: f64 = 4.0;
const OFFSET_X: f64 = 0.5;
const OFFSET_Y: f64 = 0.5;

fn main() {
    let (w, h) = (512, 512);
    // how is this not mut 🥲
    let atomic_pixels: Vec<AtomicU32> = (0..w * h).map(|_| AtomicU32::new(0)).collect();
    (0..w*h*32).into_par_iter().for_each(|i| {
        let mut rng = rand::thread_rng();
        let (y, x): (f64, f64) = (rng.gen(), rng.gen());
        let mut visited = Vec::<usize>::with_capacity(N_ITERATIONS);
        let (y, x) = ((y - OFFSET_Y) * SCALING_FACTOR, (x - OFFSET_X) * SCALING_FACTOR);

        let mut a: f64 = x;
        let mut b: f64 = y;
        for _ in 0..N_ITERATIONS {
            // z^2 + c
            // (a, b) = (a*a - b*b, 2.*a*b);

            // z^i + c = e^(i ln z) + c

            let mut distance = (a*a + b*b).sqrt().ln();
            let mut angle = b.atan2(a);
            // (distance, angle) = (distance * 2., angle * -2.);
            (distance, angle) = (distance * 2., angle * 2.);
            // (distance, angle) = (distance * 4., angle * 4.);
            // (distance, angle) = (-angle, distance);
            let distance_back = distance.exp();
            let (a_base, b_base) = (angle.cos(), angle.sin());
            (a, b) = (distance_back * a_base, distance_back * b_base);

            (a, b) = (a + x, b + y);

            let (window_x, window_y) = (a / SCALING_FACTOR + OFFSET_X, b / SCALING_FACTOR + OFFSET_Y);
            if window_x > 0. && window_x < 1. && window_y > 0. && window_y < 1. {
                let i = ((window_x * h as f64).floor() * w as f64 + (window_y * w as f64).floor()) as usize;
                let i = i.clamp(0, w * h - 1);
                visited.push(i);
            }
        }

        if (a*a + b*b) > 4. {
            for v in visited {
                atomic_pixels[v].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    });
    let pixels = atomic_pixels.iter().map(|x| x.load(std::sync::atomic::Ordering::Relaxed)).collect::<Vec<_>>();
    let max_pixel = *pixels.iter().max().unwrap();
    let pixels: Vec<[u8; 3]> = pixels.iter().map(|&p| {
        let c = p as f32 / max_pixel as f32;
        let c = (c * 255.).round() as u8;
        [c, c, c]
    }).collect();
    let out_file = File::create(Path::new("assets/out.png")).unwrap();
    let out_writer = BufWriter::new(out_file);
    let mut encoder = png::Encoder::new(out_writer, w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let data = pixels.into_iter().flatten().collect::<Vec<_>>();
    writer.write_image_data(&data).unwrap();
}
