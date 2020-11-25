use std::env;
use std::io::{self, Write};

mod mandelbrot;

enum ArgState {
    Initial,
    FocalX,
    FocalY,
    RangeWidth,
    ImageWidth,
    ImageHeight,
    Algorithm,
    Zoom,
    FrameCount,
}

enum Algorithm {
    Mandelbrot,
}

fn main() {
    let (focus_x, focus_y, mut range, width, height, algorithm, zoom, frames) = parse_args();

    // Loop through all the frames
    for _i in 0..frames {
        let header = format!("P6\n{} {}\n255\n", width, height);
        write_image(header, focus_x, focus_y, range, width, height, &algorithm);
        range *= zoom;
    }
}

fn parse_args() -> (f64, f64, f64, u32, u32, Algorithm, f64, u32) {
    let mut state = ArgState::Initial;
    let mut focus_x = -1f64;
    let mut focus_y = 0.005;
    let mut range = 0.00005;
    let mut width = 1920;
    let mut height = 1080;
    let mut algorithm = Algorithm::Mandelbrot;
    let mut zoom = 0.99;
    let mut frames = 1;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-x" | "--x_coor" => state = ArgState::FocalX,
            "-y" | "--y_coor" => state = ArgState::FocalY,
            "-r" | "--range" => state = ArgState::RangeWidth,
            "-w" | "--width" => state = ArgState::ImageWidth,
            "-ht" | "--height" => state = ArgState::ImageHeight,
            "-a" | "--algorithm" => state = ArgState::Algorithm,
            "-z" | "--zoom" => state = ArgState::Zoom,
            "-f" | "--frames" => state = ArgState::FrameCount,
            "-h" | "--help" => help_args(),
            _ => match state {
                ArgState::FocalX => focus_x = arg.parse().expect(format!("{} must be a floating point value!", arg).as_str()),
                ArgState::FocalY => focus_y = arg.parse().expect(format!("{} must be a floating point value!", arg).as_str()),
                ArgState::RangeWidth => range = arg.parse().expect(format!("{} must be a floating point value!", arg).as_str()),
                ArgState::ImageWidth => width = arg.parse().expect(format!("{} must be a positive integral value!", arg).as_str()),
                ArgState::ImageHeight => height = arg.parse().expect(format!("{} must be a positive integral value!", arg).as_str()),
                ArgState::Algorithm => {
                    match arg.as_str() {
                        "mandelbrot" | "Mandelbrot" => algorithm = Algorithm::Mandelbrot,
                        _ => {
                            eprintln!("Error unknown algorithm {}!", arg.as_str());
                            help_args();
                        },
                    }
                },
                ArgState::Zoom => zoom = arg.parse().expect(format!("{} must be a floating point value!", arg).as_str()),
                ArgState::FrameCount => frames = arg.parse().expect(format!("{} must be a positive integral value!", arg).as_str()),
                ArgState::Initial => {
                    eprintln!("Error unknown argument {}! Expected either flags -x, -y, -r, -w, -h, -a, -z, -f, or -h", arg);
                    help_args();
                },
            },
        }
    }

    (focus_x, focus_y, range, width, height, algorithm, zoom, frames)
}

fn help_args() {
    println!("usage: mandelart [options] ...");
    println!("Mandelart outputs media of a specified fractal set in netpbm PPM format.");
    println!("Media may be images (frames = 1) or video (frames >= 2).");
    println!("Media output is sent to stdout.");
    println!("-x | --x_coor         : X coordinate of the focus point of the media");
    println!("                        Default: -1");
    println!("-y | --y_coor         : Y coordinate of the focus point of the media");
    println!("                        Default: 0.005");
    println!("-r | --range          : Different between the leftmost pixel and the rightmost pixel");
    println!("                        Default: 0.00005");
    println!("-w | --width          : Number of pixels wide of the output media");
    println!("                        Default: 1920");
    println!("-ht | --height        : Number of pixels tall of the output media");
    println!("                        Default: 1080");
    println!("-a | --algorithm      : Specify which algorithm to use for fractal generation");
    println!("                        Available algorithms: mandelbrot");
    println!("                        Default: mandelbrot");
    println!("-z | --zoom           : If frames > 2, this is the zoom amount between frames");
    println!("                        Default: 0.99");
    println!("-f | --frames         : Number of images to take, zooming in to the focal point for each image");
    println!("                        Set -f to 1 for a single image, or >= 2 for a video");
    println!("                        Default: 1");
    println!("-h | --help           : See help output.  Trumps other argument flags");
    println!("\nHere is an example execution of mandelart to generate a single image of mandelbrot set using the default values:");
    println!("mandelart -x -1 -y 0.005 -r 0.00005 -w 1920 -ht 1080 -a mandelbrot -f 1");
    println!("Here is an example execution of mandelart to generate a 60 second video of mandelbrot set at 60 fps:");
    println!("mandelart -x -1 -y 0.005 -r 0.00005 -w 1920 -ht 1080 -a mandelbrot -z 0.99 -f 3600");

    std::process::exit(0);
}

fn write_image(header: String, focus_x: f64, focus_y: f64, range: f64, width: u32, height: u32, algorithm: &Algorithm) {
    let mut buffer = Vec::<u8>::new();
    buffer.extend_from_slice(header.as_bytes());
    let step_size = range / f64::from(width);
    let start_x = focus_x - (range / 2f64);
    let mut x = start_x;
    let mut y = focus_y + (step_size * f64::from(height) / 2f64);
    for _h in 0..height {
        for _w in 0..width {
            let color: (u8, u8, u8) = match algorithm {
                Algorithm::Mandelbrot => mandelbrot::color_point(x, y),
            };

            buffer.push(color.0);
            buffer.push(color.1);
            buffer.push(color.2);

            x += step_size;
        }
        x = start_x;
        y -= step_size;
    }
    buffer.push(b"\n"[0]);

    io::stdout().write_all(&buffer[..]).expect("Error: Failure to write to stdout!");
}
