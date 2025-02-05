use color_eyre::eyre::{Context, OptionExt, Report};
use raydar::{
    renderer::{cpu::CpuRenderer, vulkan::VulkanRenderer, Renderer},
    scene::Scene,
};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = Scene::default();

    let cpu_arg = std::env::args().any(|arg| arg == "--cpu");

    let mut renderer: Box<dyn Renderer> = if cpu_arg {
        Box::new(CpuRenderer::default())
    } else {
        Box::new(VulkanRenderer::new())
    };

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    println!(
        "Frame took {}ms",
        renderer
            .profiler()
            .frame_timer()
            .duration()
            .ok_or_eyre("Frame timer not started")?
            .as_millis()
    );

    Ok(())
}
