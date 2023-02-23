use image::{imageops::FilterType::Gaussian, ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::error::Error;
use std::{env, fmt};

const ASCII: &str = "@%#?+=:-. ";
// const ASCII: &str = "$@#W9876543210?!abc;:+=-,_.";

// custom error type
#[derive(Debug)]
struct ArgumentError;
impl Error for ArgumentError {}
impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Please provide path to the original image.")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // error if didn't provide file name
    if args.len() == 1 {
        println!("{}", ArgumentError);
        return Err(ArgumentError.into());
    }

    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let mut img = image::open(&args[1])?;

    // default to third of original resolution
    let scaling: f64 = if args.len() == 2 {
        0.33
    } else {
        match args[2].parse() {
            Ok(n) => n,
            Err(e) => panic!("{}", e),
        }
    };

    // constrain the greater axis
    let scaling: u32 = (scaling * std::cmp::max(img.width(), img.height()) as f64) as u32;

    img = img.resize(scaling, scaling, Gaussian);

    println!(
        "image width, height after resize: {}, {}",
        img.width(),
        img.height()
    );

    let mut output: RgbImage = ImageBuffer::new(img.width() *15, img.height() * 15);
    for row in output.rows_mut() {
        for p in row {
            p[0] = 255;
            p[1] = 255;
            p[2] = 255;
        }
    }

    let scale = Scale { x: 17., y: 17. };
    let font_data: &[u8] = include_bytes!("../consolab.ttf");
    let mut font: Font<'static> = Font::try_from_bytes(font_data).unwrap();

    let mut y: i32 = -14;
    for row in img.clone().into_luma8().rows() {
        y += 15;
        let mut x: i32 = -12;
        // print progress
        println!("{}%", y / 15 * 100 / img.height() as i32);
        for p in row {
            x += 15;

            let index = map_range((0., 255.), (0., (ASCII.len() - 1) as f64), p[0] as f64) as usize;

            let text = ASCII.chars().nth(index).unwrap().to_string();

            draw_text_mut(
                &mut output,
                Rgb([0u8, 0u8, 0u8]),
                x,
                y,
                scale,
                &mut font,
                text.as_str(),
            );
        }
    }

    println!("saving image...");
    // Write the contents of this image to the Writer in PNG format.
    output.save("output/output.png")?;
    println!("done!");
    Ok(())
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
