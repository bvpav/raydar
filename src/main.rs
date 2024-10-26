use image::{ImageBuffer, Rgba};
use rand::Rng;

fn main() {
    let (width, height) = (1024, 1024);

    let mut rng = rand::thread_rng();

    let pixels: Vec<u8> = (0..width * height)
        .map(|_| rng.gen::<u32>() | 0xff)
        .flat_map(|pixel| pixel.to_be_bytes())
        .collect();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        &pixels[..],
    )
    .unwrap();

    image.save("output.png").unwrap();
}
