use image::{
    imageops::{
        colorops::{brighten_in_place, grayscale},
        resize,
        FilterType::Gaussian,
    },
    GrayImage, ImageBuffer, Luma, Rgb, RgbImage,
};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::error::Error;
use std::{env, fmt, io::Write};

// custom error type
#[derive(Debug)]
enum ArgumentError {
    MissingArgument,
    ScaleOutOfBound,
}
impl Error for ArgumentError {}
impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgumentError::MissingArgument => {
                write!(f, "Please provide path to the original image.")
            }
            ArgumentError::ScaleOutOfBound => {
                write!(f, "The scaling factor should be between 0 and 1.")
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ascii: &str = "@%#?+=:-. ";
    // const ASCII: &str = "$@#W9876543210?!abc;:+=-,_.";

    let args: Vec<String> = env::args().collect();

    // error if didn't provide file name
    if args.len() == 1 {
        eprintln!("{}", ArgumentError::MissingArgument);
        return Err(ArgumentError::MissingArgument.into());
    }

    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let mut img = image::open(&args[1])?;

    // default to third of original resolution
    let scaling: f64 = if args.len() == 2 {
        0.33
    } else {
        match args[2].parse() {
            Ok(n) => {
                if 0. < n && n <= 1. {
                    n
                } else {
                    eprintln!("{}", ArgumentError::ScaleOutOfBound);
                    return Err(ArgumentError::ScaleOutOfBound.into());
                }
            }
            Err(e) => panic!("{}", e),
        }
    };

    // constrain the greater axis
    let scaling: u32 = (scaling * std::cmp::max(img.width(), img.height()) as f64) as u32;

    img = img.resize(scaling, scaling, Gaussian);

    // determine background color (greater the brighter)
    let original_brightness: i32 = img
        .clone()
        .resize(1, 1, Gaussian)
        .into_luma8()
        .get_pixel(0, 0)[0]
        .into();
    println!("original image brightness: {:?}", original_brightness);
    // configure color using result above
    let (mut background_color, mut text_color): (u8, u8) = (255, 0);
    if original_brightness < 110 {
        (background_color, text_color) = (0, 255);
    }

    println!(
        "image width, height after resize: {}*17, {}*17",
        img.width(),
        img.height()
    );

    let mut output: RgbImage = ImageBuffer::new(img.width() * 17, img.height() * 17);
    for row in output.rows_mut() {
        for p in row {
            p[0] = background_color;
            p[1] = background_color;
            p[2] = background_color;
        }
    }

    let scale = Scale { x: 20., y: 20. };
    let font_data: &[u8] = include_bytes!("../consolab.ttf");
    let mut font: Font<'static> = Font::try_from_bytes(font_data).unwrap();

    let mut y: i32 = -16;
    for row in img.clone().into_luma8().rows() {
        y += 17;
        let mut x: i32 = -14;

        // print progress
        print!("\r{}%", y / 17 * 100 / img.height() as i32);
        std::io::stdout().flush()?;

        for p in row {
            x += 17;

            let mut index =
                map_range((0., 255.), (0., (ascii.len() - 1) as f64), p[0] as f64) as usize;
            if original_brightness < 110 {
                index = ascii.len() - 1 - index;
            }

            let text = ascii.chars().nth(index).unwrap().to_string();

            draw_text_mut(
                &mut output,
                Rgb([text_color, text_color, text_color]),
                x,
                y,
                scale,
                &mut font,
                text.as_str(),
            );
        }
    }

    // determine output image brightness
    let output_brightness: i32 = resize(&output.clone(), 1, 1, Gaussian).get_pixel(0, 0)[0].into();
    println!("output image brightness: {:?}", output_brightness);

    // // darken or lighten output image
    brighten_in_place(
        &mut output,
        ((original_brightness as i32 - output_brightness as i32) as f32 / 1.5).floor() as i32,
    );

    println!("\nsaving image...");
    // Write the contents of this image to the Writer in PNG format.
    grayscale(&output).save("output/output.png")?;
    img.save("output/original.png")?;
    println!("done!");
    Ok(())
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
