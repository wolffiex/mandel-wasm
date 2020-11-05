use std::slice::IterMut;
use std::iter::Enumerate;
use num_complex::Complex;
use fixed::{types::extra::U123, FixedI128};
use std::ops::{Mul, Div};

type Fix = FixedI128<U123>;
type MaybeComplex = Option<Complex<Fix>>;

pub fn render<T, F>(iter: Enumerate<IterMut<T>>, store_pixel: F, step_size: i64,
                    center_x: i64, cols: usize, center_y: i64, rows: usize) where
    F: Fn(u8, u8, u8) -> T {
    let step: Fix = Fix::from_num(1).div(step_size as i128);
    let icols = cols as i64;
    let irows = rows as i64;
    let x_start = center_x - icols / 2i64;
    let y_start = center_y - irows / 2i64;
    for (uindex, pixel) in iter {
        let index = uindex as i64;
        let x_offset = (index % icols) as i64;
        let y_offset = index / icols as i64;
        if y_offset >= irows { break; }
        let cx: Fix = step.mul((x_start + x_offset) as i128);
        let cy: Fix = step.mul((y_start + y_offset) as i128);
        let z = calc_z(cx, cy);
        let (r, g, b) = map_z(z);
        *pixel = store_pixel(r, g, b);
    }
}

#[derive(Copy, Clone)]
enum ColorFn {
    Const(u8),
    Range(u8, u8),
}

const RANGES: [(u8, (ColorFn, ColorFn, ColorFn)); 2] = [
    (6, (ColorFn::Const(0), ColorFn::Const(0), ColorFn::Range(0, 250))),
    /*
    (62, (ColorFn::Range(0, 250), ColorFn::Range(0,250), ColorFn::Const(250))),
    (125, (ColorFn::Const(250), ColorFn::Const(250), ColorFn::Range(250,0))),
    (250, (ColorFn::Range(250,0), ColorFn::Const(250), ColorFn::Const(0))),
    (u8::MAX, (ColorFn::Const(250), ColorFn::Range(250,0), ColorFn::Const(0))),
     */
    (u8::MAX, (ColorFn::Range(255,0), ColorFn::Range(255,0), ColorFn::Range(255,0))),
];

fn map_color(range_info: (u8, u8), color_fn: ColorFn) -> u8 {
    match color_fn {
        ColorFn::Const(c) => c,
        ColorFn::Range(c0, c1) => {
            let (pos, steps) = range_info;
            let ci0 = c0 as i16;
            let ci1 = c1 as i16;
            let target_range: i16 = ci1 - ci0;
            let range_step = target_range/steps as i16;
            let v :i16 = ci0 + range_step * pos as i16;
            return v as u8;
        }
    }
}

fn map_z(z: u8) -> (u8, u8, u8) {
    let mut low = 0u8;
    for i in 0..RANGES.len() {
        let ( high, (r,g,b)) = RANGES[i];
        if z <= high {
            let steps = high - low;
            let range = (z - low, steps);
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
    for z in (u8::MIN..u8::MAX).step_by(5) {
        let (r, g, b) = map_z(z);
        println!("{}: ({},{}, {})", z, r, g, b)
    }
}

fn calc_z(cx: Fix, cy: Fix) -> u8 {
    let bx: Fix = Fix::from_num(cx);
    let by: Fix = Fix::from_num(cy);
    let clamp: Fix = Fix::from_num(4);
    let c = Complex::new(bx, by);

    let mut i = 0;
    let mut z = Some(c);
    while i < u8::MAX {
        //(a+biw(c+di) = (ac−bd) + (ad+bc)i
        z = iter_z(z, c);
        if !clamp_norm(z, clamp) { break; }
        i += 1;
    }
    return i;
}

fn clamp_norm(opt_lz: MaybeComplex, clamp: Fix) -> bool {
    match norm_square(opt_lz) {
        Some(n) => n < clamp,
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
    //weird but this seems to panic with negative numbers where the result overflows
    let ab = a.abs();
    ab.checked_mul(ab)
}

fn mul_safe(a: Fix, b: Fix) -> Option<Fix> {
    let negate = a.is_positive() != b.is_positive();
    let result = a.abs().checked_mul(b.abs())?;
    if negate { result.checked_neg() } else { Some(result) }
}