use clap::Parser;
use color_eyre::eyre::{Context, Report};
use std::{fs::File, io::Read, path::PathBuf};

use crate::{
    renderer::{cpu::CpuRenderer, vulkan::VulkanRenderer, Renderer, RendererConfig},
    scene::Scene,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CommonArgs {
    /// Use CPU renderer instead of Vulkan
    #[arg(long)]
    pub cpu: bool,

    /// Maximum number of samples per pixel
    #[arg(long)]
    pub max_sample_count: Option<u32>,

    /// Maximum number of light bounces
    #[arg(long)]
    pub max_bounces: Option<u32>,

    /// Path to the scene file (.rscn)
    pub scene_file: Option<PathBuf>,
}

impl CommonArgs {
    /// Initialize scene and renderer from command line arguments
    pub fn initialize(&self) -> Result<(Scene, Box<dyn Renderer>), Report> {
        let scene = if let Some(path) = &self.scene_file {
            let mut file = File::open(path).wrap_err("Cannot open scene file")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .wrap_err("Cannot read scene file")?;
            serde_json::from_str(&contents).wrap_err("Cannot parse scene file")?
        } else {
            Scene::default()
        };

        let mut config = RendererConfig::default();
        if let Some(max_sample_count) = self.max_sample_count {
            config.max_sample_count = max_sample_count;
        }
        if let Some(max_bounces) = self.max_bounces {
            config.max_bounces = max_bounces;
        }

        let renderer: Box<dyn Renderer> = if self.cpu {
            Box::new(CpuRenderer::new(config))
        } else {
            Box::new(VulkanRenderer::new(config))
        };

        Ok((scene, renderer))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct RaydarArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Output image path
    #[arg(short, long, default_value = "output.png")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct RaydarEditorArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}
