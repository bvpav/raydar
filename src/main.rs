use color_eyre::eyre::{Context, OptionExt, Report};
use owo_colors::OwoColorize;
use raydar::{
    renderer::{cpu::CpuRenderer, vulkan::VulkanRenderer, Renderer},
    scene::benchmark,
};
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let scene = if let Some(path) = std::env::args().nth(1) {
        let mut file = File::open(&path).wrap_err("Cannot open scene file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .wrap_err("Cannot read scene file")?;
        serde_json::from_str(&contents).wrap_err("Cannot parse scene file")?
    } else {
        benchmark::benchmark_scene()
    };

    let cpu_arg = std::env::args().any(|arg| arg == "--cpu");

    let mut renderer: Box<dyn Renderer> = if cpu_arg {
        Box::new(CpuRenderer::default())
    } else {
        Box::new(VulkanRenderer::default())
    };

    let image = renderer.render_frame(&scene);
    image.save("output.png").wrap_err("Cannot save image")?;

    let profiler = renderer.profiler();

    // Print all metrics with rainbow colors
    println!("\n{}", "=== Render Profiling Metrics ===".bold());

    println!(
        "{} {}ms",
        "Scene Preparation:".red().bold(),
        profiler
            .prepare_timer()
            .duration()
            .ok_or_eyre("Prepare timer not started")?
            .as_millis()
    );

    println!(
        "{} {}ms",
        "Render Time:".yellow().bold(),
        profiler
            .render_timer()
            .duration()
            .ok_or_eyre("Render timer not started")?
            .as_millis()
    );

    println!(
        "{} {}ms",
        "Last Sample Time:".green().bold(),
        profiler
            .sample_timer()
            .duration()
            .ok_or_eyre("Sample timer not started")?
            .as_millis()
    );

    println!("{}", "─".repeat(40).blue().bold());

    println!(
        "{} {}ms",
        "Total Frame Time:".magenta().bold(),
        profiler
            .frame_timer()
            .duration()
            .ok_or_eyre("Frame timer not started")?
            .as_millis()
    );

    Ok(())
}
