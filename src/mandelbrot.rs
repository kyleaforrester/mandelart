use std::cmp;
use std::convert::TryInto;

const MAX_ATTEMPTS: u32 = 4000;

pub fn color_point(x: f64, y: f64) -> (u8, u8, u8) {
    let time = escape_time(x, y);
    match time {
        Some(t) => colorize(t),
        None => (0, 0, 0),
    }
}

fn escape_time(x: f64, y: f64) -> Option<u32> {
    let mut z_real: f64 = 0f64;
    let mut z_imag: f64 = 0f64;
    for n in 0..MAX_ATTEMPTS {
        let z = recursion(z_real, z_imag, x, y);
        z_real = z.0;
        z_imag = z.1;
        if z_real.powi(2) + z_imag.powi(2) >= 4f64 {
            //print!("{},", n);
            return Some(n);
        }
    }
    None
}

fn recursion(z_real: f64, z_imag: f64, x: f64, y: f64) -> (f64, f64) {
    let new_real = z_real.powi(2) - z_imag.powi(2) + x;
    let new_imag = 2f64 * z_real * z_imag + y;
    (new_real, new_imag)
}

fn colorize(e_time: u32) -> (u8, u8, u8) {
    let half_time = e_time / 2;
    let red = cmp::min(half_time, 255);
    let green = if half_time < 256 {
        0
    } else {
        cmp::min(half_time, 255)
    };
    let blue = if half_time < 512 {
        0
    } else {
        cmp::min(half_time, 255)
    };
    (red.try_into().unwrap(), green.try_into().unwrap(), blue.try_into().unwrap())
}
