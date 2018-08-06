extern crate ansi_term;
extern crate clap;
extern crate image;
extern crate termsize;

use ansi_term::Colour::RGB;
use clap::{App, Arg};
use image::{GenericImage, Pixel, Rgb, Rgba};

mod unicode {
    pub const FULL_BLOCK: char = '█';
    pub const UPPER_HALF_BLOCK: char = '▀';
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
    fn ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    fn frame(image: &Self, screen: &Self) -> Rectangle {
        // Return a frame that would fit the image
        // height / width ration of a character
        let char_ratio = 2.1;
        if char_ratio * image.ratio() > screen.ratio() {
            Rectangle {
                width: screen.width,
                height: (screen.width as f32 / (char_ratio * image.ratio())) as u32,
            }
        } else {
            Rectangle {
                width: (screen.height as f32 * (char_ratio * image.ratio())) as u32,
                height: screen.height,
            }
        }
    }
    fn split(image: &Self, screen: &Self) -> Rectangle {
        Rectangle {
            width: image.width / screen.width,
            height: image.height / screen.height,
        }
    }
}

fn to_ansi(rgb: Rgb<u8>) -> ansi_term::Color {
    RGB(rgb[0], rgb[1], rgb[2])
}

fn average_rgb(pxs: &[(u32, u32, Rgba<u8>)]) -> Rgb<u8>
{
    let mut n = 0;
    let rgb =
        pxs.iter().map(|(_x, _y, p)| {
            n += 1;
            p.to_rgb()
        }).fold(Rgb([0, 0, 0]), |acc, x| Rgb::<usize> {
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

fn unicode_fg(unicode_char: char, dims: Rectangle) -> Box<FnMut(&(u32, u32, Rgba<u8>)) -> bool> {
    match unicode_char {
        unicode::UPPER_HALF_BLOCK => Box::new(move |(_x, y, _p)| y < &(dims.height / 2)),
        _ => Box::new(move |_| true),
    }
}

fn pixels_fitness(
    pixels: &[(u32, u32, Rgba<u8>)],
    color: Rgb<u8>,
) -> i32
{
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
    Img: GenericImage<Pixel = Rgba<u8>>
{
    let img_dims = Rectangle::from_tuple(img.dimensions());
    let mut fg_pixels = unicode_fg(unicode, img_dims);
    let fg = img.pixels().filter(|x| fg_pixels(x)).collect::<Vec<_>>();
    let bg = img.pixels().filter(|x| !fg_pixels(x)).collect::<Vec<_>>();
    let fg_color = average_rgb(&fg);
    let bg_color = average_rgb(&bg);
    let fitness = pixels_fitness(&fg, fg_color) + pixels_fitness(&bg, bg_color);
    (fitness, fg_color, bg_color) 
}

fn print_image_as_char<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img) {
    let mut unicode = unicode::FULL_BLOCK;
    let (mut fitness, mut fg, mut bg) = approximate_image_with_char(img, unicode);
    let unicode_chars = [unicode::UPPER_HALF_BLOCK];
    for unicode_char in unicode_chars.iter() {
        let (new_fitness, new_fg, new_bg) = approximate_image_with_char(img, *unicode_char);
        if new_fitness < fitness {
            fitness = new_fitness;
            fg = new_fg;
            bg = new_bg;
            unicode = *unicode_char
        }
    }
    print!(
        "{}",
        to_ansi(fg)
            .on(to_ansi(bg))
            .paint(unicode.to_string())
    )
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
    let img_dims = Rectangle::from_tuple(img.dimensions());
    let mut screen_dims = Rectangle::from_termsize();
    screen_dims = Rectangle::frame(&img_dims, &screen_dims);
    let char_dims = Rectangle::split(&img_dims, &screen_dims);

    for col in 0..screen_dims.height {
        for row in 0..screen_dims.width {
            let sub =
                img.sub_image(
                    row * char_dims.width,
                    col * char_dims.height,
                    char_dims.width,
                    char_dims.height,
                ).to_image();
            print_image_as_char(&sub);
        }
        println!("");
    }
}
