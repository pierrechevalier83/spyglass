extern crate ansi_term;
extern crate clap;
extern crate image;
extern crate termsize;

use ansi_term::{ANSIString, ANSIStrings};
use ansi_term::Colour::RGB;
use clap::{App, Arg};
use image::{GenericImage, Pixel, Rgb, Rgba};

mod unicode {
    pub(crate) const LOWER_ONE_EIGTH_BLOCK: char = '▁';
    pub(crate) const LOWER_ONE_QUARTER_BLOCK: char = '▂';
    pub(crate) const LOWER_THREE_EIGTHS_BLOCK: char = '▃';
    pub(crate) const LOWER_HALF_BLOCK: char = '▄';
    pub(crate) const LOWER_FIVE_EIGTHS_BLOCK: char = '▅';
    pub(crate) const LOWER_THREE_QUARTERS_BLOCK: char = '▆';
    pub(crate) const LOWER_SEVEN_EIGTHS_BLOCK: char = '▇';
    pub(crate) const LEFT_ONE_QUARTER_BLOCK: char = '▎';
    pub(crate) const LEFT_HALF_BLOCK: char = '▌';
    pub(crate) const LEFT_THREE_QUARTERS_BLOCK: char = '▊';
    pub(crate) const QUADRANT_LOWER_LEFT: char = '▖';
    pub(crate) const QUADRANT_LOWER_RIGHT: char = '▗';
    pub(crate) const QUADRANT_UPPER_LEFT: char = '▘';
    pub(crate) const QUADRANT_UPPER_RIGHT: char = '▝';

    pub(crate) const ALL: &[char] = &[
        LOWER_ONE_EIGTH_BLOCK,
        LOWER_ONE_QUARTER_BLOCK,
        LOWER_THREE_EIGTHS_BLOCK,
        LOWER_HALF_BLOCK,
        LOWER_FIVE_EIGTHS_BLOCK,
        LOWER_THREE_QUARTERS_BLOCK,
        LOWER_SEVEN_EIGTHS_BLOCK,
        LEFT_ONE_QUARTER_BLOCK,
        LEFT_HALF_BLOCK,
        LEFT_THREE_QUARTERS_BLOCK,
        QUADRANT_LOWER_LEFT,
        QUADRANT_LOWER_RIGHT,
        QUADRANT_UPPER_LEFT,
        QUADRANT_UPPER_RIGHT
    ];
    pub(crate) const FULL_BLOCK: char = '█';
    pub(crate) fn fg(
        unicode: char,
        dims: super::Rectangle,
    ) -> Box<FnMut(&(u32, u32, super::Rgba<u8>)) -> bool> {
        match unicode {
            LOWER_ONE_EIGTH_BLOCK => Box::new(move |(_x, y, _p)| *y > 7 * dims.width / 8),
            LOWER_ONE_QUARTER_BLOCK => Box::new(move |(_x, y, _p)| *y > 3 * dims.width / 4),
            LOWER_THREE_EIGTHS_BLOCK => Box::new(move |(_x, y, _p)| *y > 5 * dims.width / 8),
            LOWER_HALF_BLOCK => Box::new(move |(_x, y, _p)| *y > (dims.width / 2)),
            LOWER_FIVE_EIGTHS_BLOCK => Box::new(move |(_x, y, _p)| *y > 3 * dims.width / 8),
            LOWER_THREE_QUARTERS_BLOCK => Box::new(move |(_x, y, _p)| *y > dims.width / 4),
            LOWER_SEVEN_EIGTHS_BLOCK => Box::new(move |(_x, y, _p)| *y > dims.width / 8),
            LEFT_ONE_QUARTER_BLOCK => Box::new(move |(x, _y, _p)| *x < dims.height / 4),
            LEFT_HALF_BLOCK => Box::new(move |(x, _y, _p)| *x < dims.height / 2),
            LEFT_THREE_QUARTERS_BLOCK => Box::new(move |(x, _y, _p)| *x < 3 * dims.height / 4),
            QUADRANT_LOWER_LEFT => Box::new(move |(x, y, _p)| *x > dims.height/ 2 && *y < dims.width / 2),
            QUADRANT_LOWER_RIGHT => Box::new(move |(x, y, _p)| *x > dims.height/ 2 && *y > dims.width / 2),
            QUADRANT_UPPER_LEFT => Box::new(move |(x, y, _p)| *x < dims.height/ 2 && *y < dims.width / 2),
            QUADRANT_UPPER_RIGHT => Box::new(move |(x, y, _p)| *x < dims.height/ 2 && *y > dims.width / 2),
            FULL_BLOCK => Box::new(move |_| true),
            _ => Box::new(move |_| true),
        }
    }
}

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn from_tuple(tup: (u32, u32)) -> Rectangle {
        Rectangle {
            width: tup.0,
            height: tup.1,
        }
    }
    fn from_termsize() -> Rectangle {
        let dims = termsize::get().unwrap();
        Rectangle {
            width: dims.cols as u32,
            // -1 for the CLI prompt
            height: (dims.rows - 1) as u32,
        }
    }
}

fn to_ansi(rgb: Rgb<u8>) -> ansi_term::Color {
    RGB(rgb[0], rgb[1], rgb[2])
}

fn average_rgb(pxs: &[(u32, u32, Rgba<u8>)]) -> Rgb<u8> {
    let mut n = 0;
    let rgb = pxs
        .iter()
        .map(|(_x, _y, p)| {
            n += 1;
            p.to_rgb()
        })
        .fold(Rgb([0, 0, 0]), |acc, x| Rgb::<usize> {
            data: [
                acc[0] + x[0] as usize,
                acc[1] + x[1] as usize,
                acc[2] + x[2] as usize,
            ],
        });
    if n == 0 {
        Rgb([0, 0, 0])
    } else {
        Rgb([(rgb[0] / n) as u8, (rgb[1] / n) as u8, (rgb[2] / n) as u8])
    }
}

fn pixels_fitness(pixels: &[(u32, u32, Rgba<u8>)], color: Rgb<u8>) -> i32 {
    pixels
        .iter()
        .map(|(_, _, px_color)| {
            (px_color[0] as i32 - color[0] as i32).abs()
                + (px_color[1] as i32 - color[1] as i32).abs()
                + (px_color[2] as i32 - color[2] as i32).abs()
        })
        .sum()
}

fn approximate_image_with_char<Img>(img: &Img, unicode: char) -> (i32, Rgb<u8>, Rgb<u8>)
where
    Img: GenericImage<Pixel = Rgba<u8>>,
{
    let img_dims = Rectangle::from_tuple(img.dimensions());
    let mut fg_pixels = unicode::fg(unicode, img_dims);
    let fg = img.pixels().filter(|x| fg_pixels(x)).collect::<Vec<_>>();
    let bg = img.pixels().filter(|x| !fg_pixels(x)).collect::<Vec<_>>();
    let fg_color = average_rgb(&fg);
    let bg_color = average_rgb(&bg);
    let fitness = pixels_fitness(&fg, fg_color) + pixels_fitness(&bg, bg_color);
    (fitness, fg_color, bg_color)
}

fn image_as_char<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img) -> ANSIString<'static> {
    // This is no good because I do a lot of work for each character.
    // Instead, I should
    // - sort the pixels by color once,
    // - pick the median and consider the 2 bitmaps:
    // - first fg then bg and first bg then fg
    // For each char, I just have to find the bitmap that best fits any of these 2
    // I can then pick the right color with my current approximate_image_with_char function
    let mut unicode = unicode::FULL_BLOCK;
    let (mut fitness, mut fg, mut bg) = approximate_image_with_char(img, unicode);
    for unicode_char in unicode::ALL.iter() {
        let (new_fitness, new_fg, new_bg) = approximate_image_with_char(img, *unicode_char);
        if new_fitness < fitness {
            fitness = new_fitness;
            fg = new_fg;
            bg = new_bg;
            unicode = *unicode_char
        }
    }
    to_ansi(fg).on(to_ansi(bg)).paint(unicode.to_string())
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

    let char_dims = Rectangle::from_tuple((4, 8));
    let screen_dims = Rectangle::from_termsize();
    // Resize the image so it fits within the screen (preserves ratio)
    img = img.resize(screen_dims.width * char_dims.width, screen_dims.height * char_dims.height, image::FilterType::Nearest);

    let mut strings: Vec<ANSIString<'static>> = vec![];
    for col in 0..img.height() / char_dims.height {
        for row in 0..img.width() / char_dims.width {
            let sub =
                img.sub_image(
                    row * char_dims.width,
                    col * char_dims.height,
                    char_dims.width,
                    char_dims.height,
                ).to_image();
            strings.push(image_as_char(&sub));
        }
        strings.push(ansi_term::Style::new().paint("\n"));
    }
    println!("{}", &ANSIStrings(&strings));
}
