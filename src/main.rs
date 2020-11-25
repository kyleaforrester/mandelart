use std::env;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;

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
    ThreadCount,
}

#[derive(Copy, Clone)]
enum Algorithm {
    Mandelbrot,
}

fn main() {
    let (focus_x, focus_y, mut range, width, height, algorithm, zoom, frames, threads) = parse_args();

    let full_multiprocess_loops = frames/threads;
    let remainder = frames - (full_multiprocess_loops * threads);

    for _i in 0..full_multiprocess_loops {
        let mut channels = Vec::new();
        for _y in 0..threads {
            let (tx, rx) = mpsc::channel();
            channels.push(rx);

            let new_alg = algorithm.clone();
            thread::spawn(move || {
                let buffer = gen_image(focus_x, focus_y, range, width, height, new_alg);
                tx.send(buffer).unwrap();
            });
            range *= zoom;
        }

        // Gather images generated from other threads
        for rx in channels {
            let buffer = rx.recv().unwrap();
            io::stdout().write_all(&buffer[..]).expect("Error: Failure to write to stdout!");
        }
    }

    // Finish with less than full thread count
    let mut channels = Vec::new();
    for _y in 0..remainder{
        let (tx, rx) = mpsc::channel();
        channels.push(rx);

        let new_alg = algorithm.clone();
        thread::spawn(move || {
            let buffer = gen_image(focus_x, focus_y, range, width, height, new_alg);
            tx.send(buffer).unwrap();
        });
        range *= zoom;
    }

    // Gather images generated from other threads
    for rx in channels {
        let buffer = rx.recv().unwrap();
        io::stdout().write_all(&buffer[..]).expect("Error: Failure to write to stdout!");
    }

}

fn parse_args() -> (f64, f64, f64, u32, u32, Algorithm, f64, u32, u32) {
    let mut state = ArgState::Initial;
    let mut focus_x = -1f64;
    let mut focus_y = 0f64;
    let mut range = 4f64;
    let mut width = 1920;
    let mut height = 1080;
    let mut algorithm = Algorithm::Mandelbrot;
    let mut zoom = 0.99;
    let mut frames = 1;
    let mut threads = 1;
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
            "-t" | "--threads" => state = ArgState::ThreadCount,
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
                ArgState::ThreadCount => threads = arg.parse().expect(format!("{} must be a positive integral value!", arg).as_str()),
                ArgState::Initial => {
                    eprintln!("Error unknown argument {}! Expected either flags -x, -y, -r, -w, -h, -a, -z, -f, or -h", arg);
                    help_args();
                },
            },
        }
    }

    (focus_x, focus_y, range, width, height, algorithm, zoom, frames, threads)
}

fn help_args() {
    println!("usage: mandelart [options] ...");
    println!("Mandelart outputs media of a specified fractal set in netpbm PPM format.");
    println!("Media may be images (frames = 1) or video (frames >= 2).");
    println!("Media output is sent to stdout.");
    println!("-x | --x_coor         : X coordinate of the focus point of the media");
    println!("                        Default: -1");
    println!("-y | --y_coor         : Y coordinate of the focus point of the media");
    println!("                        Default: 0");
    println!("-r | --range          : Different between the leftmost pixel and the rightmost pixel");
    println!("                        Default: 4");
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
    println!("-t | --threads        : Number of threads to multiprocess the image");
    println!("                        Default: 1");
    println!("-h | --help           : See help output.  Trumps other argument flags");
    println!("\nHere is an example execution of mandelart to generate a single image of mandelbrot set:");
    println!("mandelart -x -0.754 -y 0.05 -r 0.001 -w 1920 -ht 1080 -a mandelbrot -f 1");
    println!("Here is an example execution of mandelart to generate a 60 second video of mandelbrot set at 60 fps using 6 threads:");
    println!("mandelart -x -0.754 -y 0.05 -r 0.001 -w 1920 -ht 1080 -a mandelbrot -z 0.99 -f 3600 -t 6");

    std::process::exit(0);
}

fn gen_image(focus_x: f64, focus_y: f64, range: f64, width: u32, height: u32, algorithm: Algorithm) -> Vec::<u8> {
    let mut buffer = Vec::<u8>::new();
    let header = format!("P6\n{} {}\n255\n", width, height);
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

    buffer
}
