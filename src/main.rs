use color_eyre::eyre::{Context, ContextCompat, Report};
use image::{ImageBuffer, Rgba};
use rand::Rng;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let (width, height) = (1024, 1024);

    let mut rng = rand::thread_rng();

    let pixels: Vec<u8> = (0..width * height)
        .map(|_| rng.gen::<u32>() | 0xff)
        .flat_map(|pixel| pixel.to_be_bytes())
        .collect();
    let image =
        ImageBuffer::<Rgba<u8>, _>::from_raw(width.try_into()?, height.try_into()?, &pixels[..])
            .wrap_err("Cannot create image buffer")?;

    image.save("output.png").wrap_err("Cannot save image")?;

    Ok(())
}
