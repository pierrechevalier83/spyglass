extern crate ansi_term;
extern crate clap;
extern crate image;
extern crate rusttype;
extern crate termsize;

use rusttype::{Font, FontCollection, Point, Scale};

use ansi_term::Colour::RGB;
use ansi_term::{ANSIString, ANSIStrings};
use clap::{App, Arg};
use image::{GenericImage, Pixel, Rgb, Rgba};

fn all_unicode() -> Vec<char> {
    let mut all = Vec::new();
    // ascii
    all.extend(0x0021..0x007E);
    // box drawing: 0x2500..0x257F
    all.extend(&[
        0x2501, 0x2503, 0x250F, 0x2510, 0x2517, 0x251B, 0x2523, 0x252B, 0x2522, 0x252B, 0x2533,
        0x253B, 0x254B,
    ]);
    // block elements: 0x2580..0x259F
    all.extend(0x2580..0x2591);
    all.extend(0x2594..0x259F);
    // geometric shapes: 0x25A0..0x25FF
    all.extend(&[
        0x25A0, 0x25AA, 0x25AC, 0x25AD, 0x25C6, 0x25CF, 0x25D6, 0x25D7,
    ]);
    // triangular shapes
    all.extend(&[0x25B0, 0x25B2, 0x25B6, 0x25BC, 0x25C0, 0x25C6]);
    all.extend(0x25E2..0x25E5);
    // miscellaneous symbols
    all.extend(&[0x2660, 0x2663, 0x2665, 0x2666]);
    // miscellaneous symbols and arrows
    all.extend(0x2BC0..0x2BC8);
    all.iter()
        .map(|x| std::char::from_u32(*x).unwrap())
        .collect()
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
fn test_approximate_image_with_bitmap() {
    let font = load_font();
    let half_box = char_to_bitmap(&font, '▄');
    let mut img = image::ImageBuffer::new(4, 8);
    for i in 0..4 {
        for j in 4..8 {
            img.put_pixel(i, j, Rgba([255, 255, 255, 1]))
        }
    }
    assert_eq!(
        (Rgb([255, 255, 255]), Rgb([0, 0, 0])),
        approximate_image_with_bitmap(&img, half_box)
    );
    img.put_pixel(0, 0, Rgba([255, 0, 255, 1]));
    assert_eq!(
        (Rgb([255, 255, 255]), Rgb([15, 0, 15])),
        approximate_image_with_bitmap(&img, half_box)
    );
}

fn approximate_image_with_bitmap<Img>(img: &Img, bitmap: u32) -> (Rgb<u8>, Rgb<u8>)
where
    Img: GenericImage<Pixel = Rgba<u8>>,
{
    let fg = img
        .pixels()
        .filter(|(x, y, _)| get_bit_at_index(bitmap, x + y * img.width()))
        .collect::<Vec<_>>();
    let bg = img
        .pixels()
        .filter(|(x, y, _)| !get_bit_at_index(bitmap, x + y * img.width()))
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
    let all_chars_and_bitmaps = all_chars_and_bitmaps();
    let mut img = image::ImageBuffer::new(4, 8);
    for i in 0..4 {
        for j in 4..8 {
            img.put_pixel(i, j, Rgba([255, 255, 255, 1]))
        }
    }
    assert_eq!(
        to_ansi(Rgb([255, 255, 255]))
            .on(to_ansi(Rgb([0, 0, 0])))
            .paint(half_box.to_string()),
        image_as_char(&img, &all_chars_and_bitmaps)
    );

    img.put_pixel(0, 0, Rgba([255, 0, 255, 1]));
    assert_eq!(
        to_ansi(Rgb([255, 255, 255]))
            .on(to_ansi(Rgb([15, 0, 15])))
            .paint(half_box.to_string()),
        image_as_char(&img, &all_chars_and_bitmaps)
    );
}

fn image_as_char<Img: GenericImage<Pixel = Rgba<u8>>>(
    img: &Img,
    all_chars_and_bitmaps: &[(char, u32)],
) -> ANSIString<'static> {
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

    let best_fit = all_chars_and_bitmaps
        .into_iter()
        .min_by_key(|char_bitmap| {
            std::cmp::min(
                hamming_weight(char_bitmap.1 ^ bitmap),
                hamming_weight(char_bitmap.1 ^ !bitmap),
            )
        })
        .unwrap();
    let (fg, bg) = approximate_image_with_bitmap(img, best_fit.1);
    to_ansi(fg).on(to_ansi(bg)).paint(best_fit.0.to_string())
}

#[test]
fn test_char_to_bitmap() {
    let font = load_font();
    assert_eq!(0xffffffff, char_to_bitmap(&font, '█'));
    assert_eq!(0x0000cccc, char_to_bitmap(&font, '▖'));
    assert_eq!(0xcccc0000, char_to_bitmap(&font, '▘'));
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
    assert_eq!(0x00066000, char_to_bitmap(&font, '▪'));
}

fn char_to_bitmap(font: &Font, character: char) -> u32 {
    // Render the glyph with an 8 pt font
    let mut bitmap = 0;
    let glyph = font
        .glyph(character)
        .scaled(Scale::uniform(8.))
        .positioned(Point { x: 0., y: 0. });
    let bb = glyph.pixel_bounding_box().unwrap();
    let x_starts = bb.min.x as u32;
    let y_starts = (bb.min.y + 7) as u32;
    let char_witdth = 4;
    glyph.draw(|x, y, v| {
        let index = x + x_starts + (y + y_starts) * char_witdth;
        if v > 0. {
            set_bit_at_index(&mut bitmap, index);
        }
    });
    bitmap
}

fn load_font() -> Font<'static> {
    let font_data = include_bytes!("../fonts/unifont-11.0.01.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]).unwrap_or_else(|e| {
        panic!("error constructing a FontCollection from bytes: {}", e);
    });
    let font = collection.into_font() // only succeeds if collection consists of one font
        .unwrap_or_else(|e| {
            panic!("error turning FontCollection into a Font: {}", e);
        });
    font
}

fn all_chars_and_bitmaps() -> Vec<(char, u32)> {
    let font = load_font();
    let all_chars = all_unicode();
    all_chars
        .iter()
        .map(|c| (*c, char_to_bitmap(&font, *c)))
        .collect()
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

    let all_chars_and_bitmaps = all_chars_and_bitmaps();

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
            strings.push(image_as_char(&sub, &all_chars_and_bitmaps));
        }
        strings.push(ansi_term::Style::new().paint("\n"));
    }
    println!("{}", &ANSIStrings(&strings));
}
