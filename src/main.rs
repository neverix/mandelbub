use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::atomic::AtomicU32;
use rand::Rng;
use rayon::prelude::*;


const N_ITERATIONS: [usize; 3] = [50, 25, 100];
const fn cmax(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}
const MAX_ITERATIONS: usize = cmax(cmax(N_ITERATIONS[0], N_ITERATIONS[1]), N_ITERATIONS[2]);
const ITERATE_ON_EACH_PIXEL: usize = 64;
const THRESHOLDS: [f32; 3] = [4.0, 3.0, 2.5];
const RESOLUTION: usize = 512;
const ASPECT_RATIO: f64 = 2.0;
const SCALING_FACTOR: f64 = 1.0;
const OFFSET_X: f64 = 0.0;
const OFFSET_Y: f64 = 0.5;
const MODULATION: f64 = 2.25;
const ROTATION: f64 = -std::f64::consts::FRAC_PI_2;

fn main() {
    let (w, h) = ((RESOLUTION as f64 * ASPECT_RATIO).round() as usize, RESOLUTION);
    // how is this not mut 🥲
    // let atomic_pixels: [Vec<AtomicU32>; 3] = [(0..w * h).map(|_| AtomicU32::new(0)).collect(); 3];
    let atomic_pixels: [Vec<AtomicU32>; 3] = core::array::from_fn(|_| (0..w * h).map(|_| AtomicU32::new(0)).collect());
    (0..w*h*ITERATE_ON_EACH_PIXEL).into_par_iter().for_each(|i| {
        let mut rng = rand::thread_rng();
        let (y, x): (f64, f64) = (rng.gen(), rng.gen());
        let mut visited = Vec::<(usize, usize)>::with_capacity(MAX_ITERATIONS);
        let (x, y) = ((x + OFFSET_X - 0.5) / SCALING_FACTOR * ASPECT_RATIO, (y - OFFSET_Y - 0.5) / SCALING_FACTOR);

        let mut a: f64 = x;
        let mut b: f64 = y;
        for i in 0..MAX_ITERATIONS {
            // z^2 + c
            // (a, b) = (a*a - b*b, 2.*a*b);

            // z^i + c = e^(i ln z) + c

            let mut distance = (a*a + b*b).sqrt().ln();
            let mut angle = b.atan2(a);
            // (distance, angle) = (distance * 2., angle * -2.);
            // (distance, angle) = (distance * 2., angle * 2.);
            (distance, angle) = (distance * MODULATION, angle * MODULATION);
            // (distance, angle) = (-angle, distance);
            (distance, angle) = (
                distance * ROTATION.cos() + angle * ROTATION.sin(),
                distance * -ROTATION.sin() + angle * ROTATION.cos()
                );
            let distance_back = distance.exp();
            let (a_base, b_base) = (angle.cos(), angle.sin());
            (a, b) = (distance_back * a_base, distance_back * b_base);

            (a, b) = (a + x, b + y);

            let (window_x, window_y) = (a * SCALING_FACTOR / ASPECT_RATIO - OFFSET_X + 0.5, b * SCALING_FACTOR + OFFSET_Y + 0.5);
            if window_x > 0. && window_x < 1. && window_y > 0. && window_y < 1. {
                let p = ((window_y * h as f64).floor() * w as f64 + (window_x * w as f64).floor()) as usize;
                let p = p.clamp(0, w * h - 1);
                visited.push((i, p));
            }
        }

        if (a*a + b*b) > 4. {
            for (i, v) in visited {
                for k in 0..3 {
                    if i < N_ITERATIONS[k] {
                        atomic_pixels[k][v].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }
    });
    let pixels = atomic_pixels[0].iter()
        .zip(atomic_pixels[1].iter())
        .zip(atomic_pixels[2].iter())
        .map(|((a, b), c)| [
            a.load(std::sync::atomic::Ordering::Relaxed),
            b.load(std::sync::atomic::Ordering::Relaxed),
            c.load(std::sync::atomic::Ordering::Relaxed)
        ]).collect::<Vec<_>>();
    let max_pixels = THRESHOLDS
        .iter()
        .map(|&t| ITERATE_ON_EACH_PIXEL as f32 * t)
        .collect::<Vec<_>>();
    let pixels: Vec<[u8; 3]> = pixels.iter().map(|&rgb| {
        let rgb = rgb.iter().zip(max_pixels.iter()).map(
            |(&c, &t)| (c as f32 / t).clamp(0., 1.)
        ).map(|x| (x * 255.).round() as u8).collect::<Vec<_>>();
        [rgb[0], rgb[1], rgb[2]]
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
