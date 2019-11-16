extern crate num;

use image::{png::PNGEncoder, ColorType};
use num::Complex;
use std::{fmt::Debug, fs::File, io, str::FromStr};

type Point = (usize, usize);

struct Corner {
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT");
        eprintln!(
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        );
        std::process::exit(1);
    }

    let bounds = parse_pair::<usize>(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let corner = Corner {
        upper_left,
        lower_right,
    };

    let mut pixels = vec![0; bounds.0 * bounds.1];

    render(&mut pixels, bounds, corner);

    write_image(&args[1], &pixels, bounds).expect("error writing png file")
}

fn write_image(filename: &str, pixels: &[u8], bounds: Point) -> Result<(), io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);

    let (width, height) = bounds;

    encoder.encode(&pixels, width as u32, height as u32, ColorType::Gray(8))
}

fn render(pixels: &mut [u8], bounds: Point, corner: Corner) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    let limit = 255_u32;
    let pixel_iter = (0..bounds.1).flat_map(|row| (0..bounds.0).map(move |column| (column, row)));

    pixel_iter.for_each(|(column, row)| {
        let point = pixel_to_point(&bounds, &corner, (column, row));
        pixels[row * bounds.0 + column] =
            escape_time(point, limit).map_or(0, |count| (limit - count) as u8);
    })
}

fn pixel_to_point(bounds: &Point, corner: &Corner, pixel: Point) -> Complex<f64> {
    let Corner {
        upper_left,
        lower_right,
    } = corner;

    let width = lower_right.re - upper_left.re;
    let height = upper_left.im - lower_right.im;

    let re: f64 = upper_left.re + (pixel.0 as f64 * width / bounds.0 as f64);
    let im: f64 = upper_left.im - (pixel.1 as f64 * height / bounds.1 as f64);

    Complex { re, im }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    parse_pair(s, ',').map(|(re, im)| Complex { re, im })
}

fn parse_pair<T:FromStr + Debug>(s: &str, separator: char) -> Option<(T, T)>
    where <T as std::str::FromStr>::Err : std::fmt::Debug
{
    s.find(separator).and_then(|index| {
        let (start, rest) = (&s[..index], &s[index + 1..]);

        let left = T::from_str(start).expect(&format!(
            "Error parsing `{:?}` with separator, `{:?}`",
            start,
            separator,
        ));

        let right = T::from_str(rest).expect(&format!(
            "Error parsing `{:?}` with separator, `{:?}`",
            rest,
            separator,
        ));

        Some((left, right))
    })
}

fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    (0..limit)
        .skip_while(is_in_range(Complex { re: 0.0, im: 0.0 }, c))
        .take(1)
        .last()
}

fn is_in_range<T>(mut z: Complex<f64>, c: Complex<f64>) -> impl FnMut(&T) -> bool {
    move |_| {
        z = z * z + c;
        z.norm_sqr() < 4.0
    }
}
