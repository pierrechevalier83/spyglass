extern crate ansi_term;
extern crate clap;
extern crate image;
extern crate termsize;

use ansi_term::Colour::RGB;
use clap::{App, Arg};
use image::{GenericImage, Pixel, Rgba};

mod unicode {
    pub const SQUARE: char = '█';
    pub const UPPER_HALF: char = '▀';
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

fn to_ansi(rgb: image::Rgb<u8>) -> ansi_term::Color {
    RGB(rgb[0], rgb[1], rgb[2])
}

//fn average_rgb<Pxs: Pixels<u8>>(pxs: Pxs) -> image::Rgb<u8> {
//}

fn print_image_as_char<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img) {
    let unicode_char = unicode::UPPER_HALF;
    // let fg_rgb = img.get_pixel(0, 0).to_rgb();
    let fg =
        img.pixels().filter(|(_x, y, _p)| *y < img.height() / 2);
    let n_fg = img.pixels().count() / 2;
    let fg_rgb = fg
        .map(|(_x, _y, p)| p.to_rgb())
        .fold(image::Rgb([0, 0, 0]), |acc, x| image::Rgb::<usize> {
            data: [acc[0] + x[0] as usize, acc[1] + x[1] as usize, acc[2] + x[2] as usize],
        });
    let fg_rgb = image::Rgb([(fg_rgb[0] / n_fg) as u8, (fg_rgb[1] / n_fg) as u8, (fg_rgb[2] / n_fg) as u8]);
    let bg_rgb = img.get_pixel(img.width() - 1, img.height() - 1).to_rgb();
    print!(
        "{}",
        to_ansi(fg_rgb)
            .on(to_ansi(bg_rgb))
            .paint(unicode_char.to_string())
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
