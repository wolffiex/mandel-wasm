extern crate image;

use image::{ImageBuffer, Rgba};

#[path = "../mandelbrot.rs"]
mod mandelbrot;

fn main() -> Result<(), std::io::Error> {
    // Create a new ImgBuf with width: imgx and height: imgy
    let cols = 500;
    let rows = 400;

    let mut img_buf = vec![Rgba([0, 0, 0, 0]); rows * cols];

    let x= -340;
    let y= -200;
    let step_size= 160;
    //let center_x= 68320;
    //let center_y= 36990;
    //let step_size=180000;

    mandelbrot::render(img_buf.iter_mut().enumerate(), store_pixel,
                       step_size, x, cols, y, rows);

    let mut img: image::ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(cols as u32, rows as u32);
    // it seems silly we have to do this but I couldn't get IterMut and PixelsMut to line up, type-wise
    for (index, pixel) in img.pixels_mut().enumerate() {
        *pixel = img_buf[index];
    }
    img.save("target/fractal.png").unwrap();
    Ok(())
}

fn store_pixel(r: u8, g: u8, b: u8) -> Rgba<u8> {
    image::Rgba([r, g, b, u8::MAX])
}

