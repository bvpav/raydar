use color_eyre::eyre::{Context, Report};
use raydar::{
    renderer::{vulkan::VulkanRenderer, Renderer},
    scene::Scene,
};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = Scene::default();

    // let mut renderer = CpuRenderer::default();
    let mut renderer = VulkanRenderer::new();

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    println!(
        "Frame took {}ms",
        renderer.timer().last_frame_duration().unwrap().as_millis()
    );

    Ok(())
}
