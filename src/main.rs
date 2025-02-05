use clap::Parser;
use color_eyre::eyre::{self, Context, OptionExt};
use owo_colors::OwoColorize;
use raydar::cli::RaydarArgs;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = RaydarArgs::parse();
    let (scene, mut renderer) = args.common.initialize()?;

    let image = renderer.render_frame(&scene);
    image.save(&args.output).wrap_err("Cannot save image")?;

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
