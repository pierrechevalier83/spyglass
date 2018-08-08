extern crate ansi_term;
extern crate clap;
extern crate font_rs;
extern crate image;
extern crate termsize;

use ansi_term::Colour::RGB;
use ansi_term::{ANSIString, ANSIStrings};
use clap::{App, Arg};
use font_rs::font::Font;
use image::{GenericImage, Pixel, Rgb, Rgba};
use std::fs::File;
use std::io::Read;

fn all_unicode() -> Vec<char> {
    vec!('▁', '▂', '▃', '▄', '▅', '▆', '▇', '▎', '▌', '▊', '▖', '▗', '▘', '▝')
}

#[test]
fn test_get_bit_at_index() {
    let lower_half = 0x0000ffff;
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

#[test]
fn test_approximate_image_with_char() {
    let half_box = '▄';
    let mut img = image::ImageBuffer::new(4, 8);
    for i in 0..4 {
        for j in 4..8 {
            img.put_pixel(i, j, Rgba([255, 255, 255, 1]))
        }
    }
    let font_data = font_data();
    let font = font_rs::font::parse(&font_data).unwrap();
    assert_eq!(
        (Rgb([255, 255, 255]), Rgb([0, 0, 0])),
        approximate_image_with_char(&img, &half_box, &font)
    );
    img.put_pixel(0, 0, Rgba([255, 0, 255, 1]));
    assert_eq!(
        (Rgb([255, 255, 255]), Rgb([15, 0, 15])),
        approximate_image_with_char(&img, &half_box, &font)
    );
}

fn approximate_image_with_char<Img>(img: &Img, unicode: &char, font: &Font) -> (Rgb<u8>, Rgb<u8>)
where
    Img: GenericImage<Pixel = Rgba<u8>>,
{
    let fg_pixels = char_to_bitmap(font, *unicode);
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

#[test]
fn test_image_as_char() {
    let half_box = '▄';
    let mut img = image::ImageBuffer::new(4, 8);
    for i in 0..4 {
        for j in 4..8 {
            img.put_pixel(i, j, Rgba([255, 255, 255, 1]))
        }
    }
    let font_data = font_data();
    let font = font_rs::font::parse(&font_data).unwrap();
    assert_eq!(
        to_ansi(Rgb([255, 255, 255]))
            .on(to_ansi(Rgb([0, 0, 0])))
            .paint(half_box.to_string()),
        image_as_char(&img, &font)
    );
    img.put_pixel(0, 0, Rgba([255, 0, 255, 1]));
    assert_eq!(
        to_ansi(Rgb([255, 255, 255]))
            .on(to_ansi(Rgb([15, 0, 15])))
            .paint(half_box.to_string()),
        image_as_char(&img, &font)
    );
}

fn image_as_char<Img: GenericImage<Pixel = Rgba<u8>>>(img: &Img, font: &Font) -> ANSIString<'static> {
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

    let all_characters = all_unicode();
    let best_fit = all_characters
        .into_iter()
        .min_by_key(|c| {
            std::cmp::min(
                hamming_weight(char_to_bitmap(font, *c) ^ bitmap),
                hamming_weight(char_to_bitmap(font, *c) ^ !bitmap),
            )
        })
        .unwrap();
    let (fg, bg) = approximate_image_with_char(img, &best_fit, &font);
    to_ansi(fg)
        .on(to_ansi(bg))
        .paint(best_fit.to_string())
}

#[test]
fn test_char_to_bitmap() {
    let mut font_file = File::open("fonts/SourceCodePro-Black.ttf").unwrap();
    let mut data = Vec::new();
    font_file.read_to_end(&mut data).unwrap();
    let font = font_rs::font::parse(&data).unwrap();
    assert_eq!(0x0000cccc, char_to_bitmap(&font, '▖'));
    assert_eq!(0x00003333, char_to_bitmap(&font, '▗'));
    assert_eq!(0xcccc0000, char_to_bitmap(&font, '▘'));
    assert_eq!(0x33330000, char_to_bitmap(&font, '▝'));
    assert_eq!(0x0000000f, char_to_bitmap(&font, '▁'));
    assert_eq!(0x000000ff, char_to_bitmap(&font, '▂'));
    assert_eq!(0x00000fff, char_to_bitmap(&font, '▃'));
    assert_eq!(0x0000ffff, char_to_bitmap(&font, '▄'));
    assert_eq!(0x000fffff, char_to_bitmap(&font, '▅'));
    assert_eq!(0x00ffffff, char_to_bitmap(&font, '▆'));
    assert_eq!(0x0fffffff, char_to_bitmap(&font, '▇'));
    assert_eq!(0x88888888, char_to_bitmap(&font, '▎'));
    assert_eq!(0xcccccccc, char_to_bitmap(&font, '▌'));
    assert_eq!(0xeeeeeeee, char_to_bitmap(&font, '▊'));
}

fn char_to_bitmap(font: &Font, character: char) -> u32 {
    // Render the glyph with an 8 pt font
    let glyph_bitmap = font
        .render_glyph(font.lookup_glyph_id(character as u32).unwrap(), 8)
        .unwrap();
    let mut bitmap = 0;
    // skip 1 row and 1 col because there is always a shading area that's
    // 1 pixel on the top and on the right.
    let (w, h) = (4, 8);
    let first_row_index = (h + glyph_bitmap.top - 1) as usize;
    glyph_bitmap.data.chunks(glyph_bitmap.width).skip(1).enumerate().for_each(|(x, row)| {
        let x = first_row_index + x;
        row.iter().skip(1).enumerate().for_each(|(y, color)| {
            let index = w * ( x as i32) + y as i32 + glyph_bitmap.left as i32;
            if *color > 0 && index < 32 {
                set_bit_at_index(&mut bitmap, index as u32);
            }
        });
    });
    bitmap
}

fn font_data() -> Vec<u8> {
    let mut font_file = File::open("fonts/SourceCodePro-Regular.ttf").unwrap();
    let mut data = Vec::new();
    font_file.read_to_end(&mut data).unwrap();
    data
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

    let font_data = font_data();
    let font = font_rs::font::parse(&font_data).unwrap();

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
            strings.push(image_as_char(&sub, &font));
        }
        strings.push(ansi_term::Style::new().paint("\n"));
    }
    println!("{}", &ANSIStrings(&strings));
}
