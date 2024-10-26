use color_eyre::eyre::{Context, Report};
use image::{ImageBuffer, Rgba};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let (width, height) = (256, 256);

    let mut image = ImageBuffer::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let r = (x as f32 / width as f32 * 255.0) as u8;
        let g = (y as f32 / height as f32 * 255.0) as u8;
        let b = 0;
        *pixel = Rgba([r, g, b, 255]);
    }

    image.save("output.png").wrap_err("Cannot save image")?;

    Ok(())
}
