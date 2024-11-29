use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    let (w, h) = (512, 512);
    let pixels = (0..w * h).map(|i| {
        let (y, x) = (i / h, i % h);
        if 2 * x + y < 300 {
            (100, 200, 5)
        } else {
            (200, 200, 5)
        }
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
