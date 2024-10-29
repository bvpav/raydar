use cgmath::{Deg, Point3, Vector3};
use color_eyre::eyre::{Context, Report};
use raydar::{
    renderer::Renderer,
    scene::{
        camera::{Camera, Projection},
        objects::Sphere,
        Scene,
    },
};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = Scene {
        camera: Camera::new(
            Point3::new(0.0, 0.0, -1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
            854,
            480,
            0.01,
            1000.0,
            Projection::Perspective { fov: Deg(90.0) },
        ),
        sphere: Sphere {
            center: Point3::new(0.0, 0.0, 0.0),
            radius: 0.5,
        },
    };

    let mut renderer = Renderer::default();

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    println!(
        "Frame took {}ms",
        renderer.last_frame_duration.unwrap().as_millis()
    );

    Ok(())
}
