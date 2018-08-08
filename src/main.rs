extern crate ansi_term;
extern crate clap;
extern crate image;
extern crate termsize;

use ansi_term::Colour::RGB;
use ansi_term::{ANSIString, ANSIStrings};
use clap::{App, Arg};
use image::{GenericImage, Pixel, Rgb, Rgba};

enum Unicode {
    Space,
    LowerOneEigthBlock,
    LowerOneQuarterBlock,
    LowerThreeEigthsBlock,
    LowerHalfBlock,
    LowerFiveEigthsBlock,
    LowerThreeQuartersBlock,
    LowerSevenEigthsBlock,
    LeftOneQuarterBlock,
    LeftHalfBlock,
    LeftThreeQuartersBlock,
    QuadrantLowerLeft,
    QuadrantLowerRight,
    QuadrantUpperLeft,
    QuadrantUpperRight,
}

impl Unicode {
    fn all() -> Vec<Unicode> {
        vec![
            Unicode::Space,
            Unicode::LowerOneEigthBlock,
            Unicode::LowerOneQuarterBlock,
            Unicode::LowerThreeEigthsBlock,
            Unicode::LowerHalfBlock,
            Unicode::LowerFiveEigthsBlock,
            Unicode::LowerThreeQuartersBlock,
            Unicode::LowerSevenEigthsBlock,
            Unicode::LeftOneQuarterBlock,
            Unicode::LeftHalfBlock,
            Unicode::LeftThreeQuartersBlock,
            Unicode::QuadrantLowerLeft,
            Unicode::QuadrantLowerRight,
            Unicode::QuadrantUpperLeft,
            Unicode::QuadrantUpperRight,
        ]
    }
    fn character(&self) -> char {
        match self {
            Unicode::Space => ' ',
            Unicode::LowerOneEigthBlock => '▁',
            Unicode::LowerOneQuarterBlock => '▂',
            Unicode::LowerThreeEigthsBlock => '▃',
            Unicode::LowerHalfBlock => '▄',
            Unicode::LowerFiveEigthsBlock => '▅',
            Unicode::LowerThreeQuartersBlock => '▆',
            Unicode::LowerSevenEigthsBlock => '▇',
            Unicode::LeftOneQuarterBlock => '▎',
            Unicode::LeftHalfBlock => '▌',
            Unicode::LeftThreeQuartersBlock => '▊',
            Unicode::QuadrantLowerLeft => '▖',
            Unicode::QuadrantLowerRight => '▗',
            Unicode::QuadrantUpperLeft => '▘',
            Unicode::QuadrantUpperRight => '▝',
        }
    }
    fn bitmap(&self) -> u32 {
        match self {
            Unicode::Space => 0x00000000,
            Unicode::LowerOneEigthBlock => 0x0000000f,
            Unicode::LowerOneQuarterBlock => 0x000000ff,
            Unicode::LowerThreeEigthsBlock => 0x00000fff,
            Unicode::LowerHalfBlock => 0x0000ffff,
            Unicode::LowerFiveEigthsBlock => 0x000fffff,
            Unicode::LowerThreeQuartersBlock => 0x00ffffff,
            Unicode::LowerSevenEigthsBlock => 0x07ffffff,
            Unicode::LeftOneQuarterBlock => 0x88888888,
            Unicode::LeftHalfBlock => 0xcccccccc,
            Unicode::LeftThreeQuartersBlock => 0xeeeeeeee,
            Unicode::QuadrantLowerLeft => 0x0000cccc,
            Unicode::QuadrantLowerRight => 0x00003333,
            Unicode::QuadrantUpperLeft => 0xcccc0000,
            Unicode::QuadrantUpperRight => 0x33330000,
        }
    }
}

#[test] fn test_get_bit_at_index() {
    let lower_half = Unicode::LowerHalfBlock.bitmap();
    assert_eq!(false, get_bit_at_index(lower_half, 0));
    assert_eq!(false, get_bit_at_index(lower_half, 1));
    assert_eq!(false, get_bit_at_index(lower_half, 15));
    assert_eq!(true, get_bit_at_index(lower_half, 16));
    assert_eq!(true, get_bit_at_index(lower_half, 31));
}

fn get_bit_at_index(bitmap: u32, index: u32) -> bool {
    // Reading bits from left to right,
    // so index 0 == most significant digit
    (bitmap & (1 << (31 - index))) != 0
}

fn set_bit_at_index(bitmap: &mut u32, index: u32) {
    let bit = 1 << (31 - index);
    *bitmap |= bit;
}

// The number of bits set
fn hamming_weight(val: u32) -> u32 {
    let v1 = val - ((val >> 1) & 0x55555555);
    let v2 = (v1 & 0x33333333) + ((v1 >> 2) & 0x33333333);
    (((v2 + (v2 >> 4)) & 0xF0F0F0F).wrapping_mul(0x1010101)) >> 24
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

#[test] fn test_approximate_image_with_char() {
    let half_box = Unicode::LowerHalfBlock;
    let mut img = image::ImageBuffer::new(4, 8);
    for i in 0..4 {
        for j in 4..8 {
            img.put_pixel(i, j, Rgba([255, 255, 255, 1]))
        }
    }
    assert_eq!((Rgb([255, 255, 255]), Rgb([0, 0, 0])), approximate_image_with_char(&img, &half_box));
    img.put_pixel(0, 0, Rgba([255, 0, 255, 1]));
    assert_eq!((Rgb([255, 255, 255]), Rgb([15, 0, 15])), approximate_image_with_char(&img, &half_box));
}

fn approximate_image_with_char<Img>(img: &Img, unicode: &Unicode) -> (Rgb<u8>, Rgb<u8>)
where
    Img: GenericImage<Pixel = Rgba<u8>>,
{
    let fg_pixels = unicode.bitmap();
    let fg = img
        .pixels()
        .filter(|(x, y, _)| get_bit_at_index(fg_pixels, x + y * img.width()))
        .collect::<Vec<_>>();
    let bg = img
        .pixels()
        .filter(|(x, y, _)| !get_bit_at_index(fg_pixels, x + y * img.width()))
        .collect::<Vec<_>>();
    let fg_color = average_rgb(&fg);
    let bg_color = average_rgb(&bg);
    (fg_color, bg_color)
}

fn min_by_channel<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img, channel: usize) -> u8 {
    img.pixels().map(|(_, _, px)| px[channel]).min().unwrap()
}
fn max_by_channel<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img, channel: usize) -> u8 {
    img.pixels().map(|(_, _, px)| px[channel]).max().unwrap()
}

fn image_as_char<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img) -> ANSIString<'static> {
    let (channel, (min, max)) = (0..3)
        .map(|channel| (min_by_channel(img, channel), max_by_channel(img, channel)))
        .enumerate()
        .max_by_key(|(_, (min, max))| max - min)
        .unwrap();
    let split_value = min + (max - min) / 2;
    let mut bitmap: u32 = 0;
    img.pixels()
        .filter(|(_, _, p)| p[channel] < split_value)
        .for_each(|(x, y, _)| {
            set_bit_at_index(&mut bitmap, x + y * img.width());
        });

    let all_characters = Unicode::all();
    let best_fit = all_characters
        .iter()
        .min_by_key(|c| {
            std::cmp::min(
                hamming_weight(c.bitmap() ^ bitmap),
                hamming_weight(c.bitmap() ^ !bitmap),
            )
        })
        .unwrap();
    let (fg, bg) = approximate_image_with_char(img, best_fit);
    to_ansi(fg)
        .on(to_ansi(bg))
        .paint(best_fit.character().to_string())
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
    img = img.resize(
        screen_dims.width * char_dims.width,
        screen_dims.height * char_dims.height,
        image::FilterType::Nearest,
    );

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
