use std::slice::IterMut;
use std::iter::Enumerate;
use num_complex::Complex;
use fixed::{types::extra::U123, FixedI128};

type Fix = FixedI128<U123>;
type MaybeComplex = Option<Complex<Fix>>;

const MAX_ITER: usize = 500;
pub fn render<T, F>(iter: Enumerate<IterMut<T>>, store_pixel: F, step_size: i128,
                    start_x: i128, cols: usize, start_y: i128, rows: usize) where
    F: Fn(u8, u8, u8) -> T {
    let step: Fix = Fix::from_num(1)/step_size;

    for (index, pixel) in iter {
        let x = index % cols;
        let y = index / cols;
        if y >= rows { break; }
        let x_proj = step * (start_x + x as i128);
        let y_proj = step * (start_y + y as i128);
        let z = calc_z(x_proj, y_proj);
        let (r, g, b) = map_z(z);
        *pixel = store_pixel(r, g, b);
    }
}


#[derive(Copy, Clone)]
enum ColorFn {
    Const(u8),
    Range(u8, u8),
}

const RANGES: [(usize, (ColorFn, ColorFn, ColorFn)); 4] = [
    (10, (ColorFn::Const(0), ColorFn::Const(0), ColorFn::Range(0, 250))),
    (200, (ColorFn::Range(0,255), ColorFn::Range(0, 255), ColorFn::Const(255))),
    (350, (ColorFn::Range(255,0), ColorFn::Const(255), ColorFn::Range(255,0))),
    (MAX_ITER, (ColorFn::Const(0), ColorFn::Range(255,0), ColorFn::Const(0))),
];

fn map_color(range_info: (usize, usize), color_fn: ColorFn) -> u8 {
    match color_fn {
        ColorFn::Const(c) => c,
        ColorFn::Range(c0, c1) => {
            let (pos, steps) = range_info;
            let ci0 = c0 as f64;
            let ci1 = c1 as f64;
            let target_range: f64 = ci1 - ci0;
            let range_step :f64 = target_range/steps as f64;
            //println!("target {} rangestep {}", target_range, range_step);
            let v :f64 = ci0 + range_step * pos as f64;
            return v as u8;
        }
    }
}

fn map_z(z: usize) -> (u8, u8, u8) {
    let mut low = 0;
    for i in 0..RANGES.len() {
        let ( high, (r,g,b)) = RANGES[i];
        if z <= high {
            let steps = high - low;
            let range = (z - low, steps);
            //println!("steps {} pos {}", steps, z-low);
            return (map_color(range, r),
                    map_color(range,g),
                    map_color(range,b));
        }
        low = high;
    }
    //this should be unreachable
    (0, 0, 0)
}


#[test]
fn test_render_map() {
    for z in (5..MAX_ITER).step_by(25) {
        let (r, g, b) = map_z(z);
        println!("{}: ({},{}, {})", z, r, g, b)
    }
}

fn calc_z(re: Fix, im: Fix) -> usize {
    let clamp: Fix = Fix::from_num(2);
    let c = Complex::new(re, im);

    let mut i = 0;
    let mut z = Some(c);
    while i < MAX_ITER {
        //(a+biw(c+di) = (ac−bd) + (ad+bc)i
        z = iter_z(z, c);
        if !clamp_norm(z, clamp) { break; }
        i += 1;
    }
    return i;
}

fn clamp_norm(opt_lz: MaybeComplex, clamp: Fix) -> bool {
    match norm_square(opt_lz) {
        Some(n) => n <= clamp,
        None => false
    }
}

fn iter_z(opt_lz: MaybeComplex, c: Complex<Fix>) -> MaybeComplex {
    let lz = opt_lz?;
    let sqr_re = sq_safe(lz.re)?.checked_sub(sq_safe(lz.im)?)?;
    let sqr_im = mul_safe(lz.re, mul_safe(lz.im, Fix::from_num(2))?)?;
    Some(Complex::new(sqr_re.checked_add(c.re)?, sqr_im.checked_add(c.im)?))
}

fn norm_square(o: MaybeComplex) -> Option<Fix> {
    let lz = o?;
    let im = mul_safe(lz.im, lz.im)?;
    sq_safe(lz.re)?.checked_add(im)
}

fn sq_safe(a: Fix) -> Option<Fix> {
    mul_safe(a, a)
}

fn mul_safe(a: Fix, b: Fix) -> Option<Fix> {
    a.checked_mul(b)
}