extern crate ansi_term;
extern crate clap;
extern crate image;
extern crate termsize;

use ansi_term::Colour::RGB;
use clap::{App, Arg};
use image::{GenericImage, Pixel};

mod unicode {
    pub const SQUARE: char = '█';
}

fn main() {
    let matches = App::new("spyglass")
        .version("0.1")
        .about("renders an image into unicode")
        .author("Pierre Chevalier")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    let img_path = matches.value_of("INPUT").unwrap();
    let mut img = image::open(img_path).unwrap();
    let img_dims = img.dimensions();
    let dims = termsize::get().unwrap();

    let char_width = img_dims.0 / (dims.rows - 1) as u32;
    let char_height = img_dims.1 / dims.cols as u32;
    println!("{}, {}", char_width, char_height);

    for row in 0..dims.rows as u32 - 1 {
        for col in 0..dims.cols as u32 {
            let sub = img.sub_image(row * char_width, col * char_height, char_width, char_height).to_image();
            let rgb = sub.get_pixel(0, 0).to_rgb();
            print!("{}", RGB(rgb[0], rgb[1], rgb[2]).paint(unicode::SQUARE.to_string()))
        }
        println!("");
    }
}
