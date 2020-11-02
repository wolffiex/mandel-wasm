use std::slice::IterMut;
use std::iter::Enumerate;
use num_traits;
use num_rational::Rational64;
use num_traits::ToPrimitive;

pub fn render<T, F>(iter: Enumerate<IterMut<T>>, store_pixel: F, cols: usize, rows: usize) where
    F: Fn(u8, u8, u8) ->  T {
    let denom = 180 as i64;
    let x_start = -2 * denom;
    let y_start = denom * -1;
    for (index, pixel) in iter {
        let cxr = Rational64::new(x_start + (index%cols) as i64, denom);
        //let cxr = x_start + Rational64::new(3*(index % cols) as i64, 500 as i64);
        //let cx = -2.0 + step_size * (index % cols) as f64;
        let cx = cxr.to_f64().unwrap();
        let y_offset = index / cols;
        if y_offset >= rows {
            break;
        }
        let cyr = Rational64::new(y_start + y_offset as i64, denom);
        //let cxr = x_start + Rational64::new(3*(index % cols) as i64, 500 as i64);
        //let cy = -1.0 + y_offset as f64 * 0.006;
        let cy = cyr.to_f64().unwrap();
        let z = calc_z(cx, cy, 3.0);
        *pixel = store_pixel(z, z, z);
    }
}


fn calc_z<T: Copy + num_traits::Num + std::cmp::PartialOrd>(cx: T, cy: T, clamp: T) -> u8 {
    let c = num_complex::Complex::new(cx, cy);
    let mut z = num_complex::Complex::new(cx, cy);

    let mut i = 0;
    while i < u8::MAX && z.norm_sqr() < clamp {
        z = z * z + c;
        i += 1;
    }
    return i;
}
