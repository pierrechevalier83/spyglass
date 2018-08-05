extern crate ansi_term;
extern crate clap;
extern crate image;

use clap::{App, Arg};
use image::GenericImage;

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
    let img = image::open(img_path).unwrap();
    println!("dimensions {:?}", img.dimensions());
    println!(
        "{}",
        ansi_term::Colour::RGB(70, 130, 180).paint(unicode::SQUARE.to_string())
    )
}
