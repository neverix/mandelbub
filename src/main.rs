use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn main() {
    let (w, h) = (512, 512);
    let out_file = File::create(Path::new("assets/out.png")).unwrap();
    let out_writer = BufWriter::new(out_file);
    let mut encoder = png::Encoder::new(out_writer, w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let data = [0u8; 512*512*3];
    writer.write_image_data(&data).unwrap();
}
