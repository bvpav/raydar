use cgmath::Point3;
use color_eyre::eyre::{Context, Report};
use raydar::*;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = Scene {
        resolution_x: 854,
        resolution_y: 480,
        sphere: Sphere {
            center: Point3::new(0.0, 0.0, 0.0),
            radius: 0.5,
        },
    };

    let renderer = Renderer;

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    Ok(())
}
