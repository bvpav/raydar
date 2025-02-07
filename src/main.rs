use clap::{crate_version, Parser};
use color_eyre::eyre::{self, Context, OptionExt};
use owo_colors::OwoColorize;
use raydar::{
    cli::RaydarArgs,
    renderer::{timing::Profiler, Renderer},
    scene::Scene,
};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = RaydarArgs::parse();
    let (scene, mut renderer) = args.common.initialize()?;

    print_info(&args, &scene, renderer.as_ref());

    let image = renderer.render_frame(&scene);
    image.save(&args.output).wrap_err("Cannot save image")?;

    print_profiling_metrics(renderer.profiler())?;

    Ok(())
}

fn print_info(args: &RaydarArgs, scene: &Scene, renderer: &dyn Renderer) {
    println!(
        "{}",
        format!("{}", format!("=== Raydar v{} ===", crate_version!()).bold())
    );

    println!(
        "{} {}",
        "Renderer:".red().bold(),
        if args.common.cpu { "CPU" } else { "Vulkan" }
    );

    println!(
        "{} {}",
        "Max Samples:".yellow().bold(),
        renderer.max_sample_count()
    );

    println!(
        "{} {}",
        "Max Bounces:".green().bold(),
        renderer.max_bounces()
    );

    println!(
        "{} {}x{}",
        "Resolution:".blue().bold(),
        scene.camera.resolution_x(),
        scene.camera.resolution_y()
    );

    println!("{} {}", "Objects:".magenta().bold(), scene.objects.len());
}

fn print_profiling_metrics(profiler: &Profiler) -> eyre::Result<()> {
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
