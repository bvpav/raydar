use color_eyre::eyre::{Context, Report};
use raydar::{renderer::Renderer, scene::Scene};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = Scene::default();

    let mut renderer = Renderer::default();

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    println!(
        "Frame took {}ms",
        renderer.last_frame_duration.unwrap().as_millis()
    );

    Ok(())
}
