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
        show_proper_usage(&args);
    }

    let bounds = parse_pair::<usize>(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    let (width, height) = bounds;
    let mut pixels = vec![0; width * height];

    render(
        &mut pixels,
        bounds,
        Corner {
            upper_left,
            lower_right,
        },
    );

    write_image(&args[1], &pixels, bounds).expect("error writing png file")
}

fn show_proper_usage(args: &[String]) {
    eprintln!("Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT");
    eprintln!(
        "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
        args[0]
    );
    std::process::exit(1);
}

fn write_image(filename: &str, pixels: &[u8], (width, height): Point) -> Result<(), io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);

    encoder.encode(&pixels, width as u32, height as u32, ColorType::Gray(8))
}

fn render(pixels: &mut [u8], bounds: Point, corner: Corner) {
    let (width, height) = bounds;
    assert!(pixels.len() == width * height);

    let limit = 255_u32;
    let pixel_iter = (0..height).flat_map(|row| (0..width).map(move |column| (column, row)));

    pixel_iter.for_each(|point| {
        let (column, row) = point;
        let point = pixel_to_point(&bounds, &corner, point);

        pixels[row * width + column] =
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

    let re = upper_left.re + (pixel.0 as f64 * width / bounds.0 as f64);
    let im = upper_left.im - (pixel.1 as f64 * height / bounds.1 as f64);

    Complex { re, im }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    parse_pair(s, ',').map(|(re, im)| Complex { re, im })
}

fn parse_pair<T: FromStr + Debug>(s: &str, separator: char) -> Option<(T, T)> {
    s.find(separator).map(|index| {
        let (start, rest) = split_at_exlusive(s, index);

        (
            from_str_or(start, parse_exit(start, s, separator)),
            from_str_or(rest, parse_exit(rest, s, separator)),
        )
    })
}

fn split_at_exlusive(s: &str, index: usize) -> (&str, &str) {
    (&s[..index], &s[index + 1..])
}

fn from_str_or<T: FromStr + Debug, F>(part: &str, f: F) -> T
where
    F: FnOnce(T::Err) -> T,
{
    T::from_str(part).unwrap_or_else(f)
}

fn parse_exit<'a, T, E>(sub: &'a str, s: &'a str, separator: char) -> impl FnOnce(E) -> T + 'a {
    move |_| {
        eprintln!(
            "Error parsing {:?} from {:?} with separator, \"{}\"",
            sub, s, separator,
        );

        std::process::exit(1)
    }
}

fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    (0..limit).skip_while(is_in_range(c)).take(1).last()
}

fn is_in_range<T>(c: Complex<f64>) -> impl FnMut(&T) -> bool {
    let mut z = Complex { re: 0.0, im: 0.0 };
    move |_| {
        z = z * z + c;
        z.norm_sqr() < 4.0
    }
}
