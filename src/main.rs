//! everwall — recolor any wallpaper into the Everforest palette.

mod color;
mod palette;
mod recolor;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use palette::Variant;
use recolor::Options;

/// Recolor any image into the Everforest palette.
#[derive(Parser, Debug)]
#[command(name = "everwall", version, about, long_about = None)]
struct Cli {
    /// Input image (png, jpeg, webp, bmp, tiff, ...).
    #[arg(required_unless_present = "list_palettes")]
    input: Option<PathBuf>,

    /// Output path. Defaults to "<input>-<palette>.png" next to the input.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Which Everforest variant to theme toward.
    #[arg(short, long, value_enum, default_value_t = Variant::DarkMedium)]
    palette: Variant,

    /// How many nearest palette colors to blend per pixel (1 = hard snap).
    #[arg(short, long, default_value_t = 8)]
    nearest: usize,

    /// Gaussian blend falloff. Higher = crisper/more posterized, lower = softer.
    #[arg(short, long, default_value_t = 64.0)]
    shape: f32,

    /// Keep the original lightness; let the palette drive only hue and chroma.
    /// Preserves fine detail — good for photographic wallpapers.
    #[arg(short = 'L', long)]
    preserve_luminance: bool,

    /// Also use Everforest's muted accent-tinted backgrounds as anchors. Adds
    /// colored shadows at the cost of a slightly pinker/less-green cast.
    #[arg(short = 'r', long)]
    rich: bool,

    /// List the available palette variants and exit.
    #[arg(long)]
    list_palettes: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_palettes {
        println!("Available Everforest palettes:");
        for v in Variant::ALL {
            println!("  {}", v.name());
        }
        return Ok(());
    }

    let input = cli.input.expect("clap requires input unless --list-palettes");
    let output = cli.output.unwrap_or_else(|| default_output(&input, cli.palette));

    let palette = cli.palette.palette(cli.rich);
    let opts = Options {
        nearest: cli.nearest,
        shape: cli.shape,
        preserve_luminance: cli.preserve_luminance,
    };

    eprintln!(
        "everwall: {} -> {}  [{}, {} colors, nearest={}, shape={}{}]",
        input.display(),
        output.display(),
        palette.variant.name(),
        palette.colors.len(),
        opts.nearest,
        opts.shape,
        if opts.preserve_luminance {
            ", preserve-luminance"
        } else {
            ""
        },
    );

    let mut img = image::open(&input)
        .with_context(|| format!("failed to open image: {}", input.display()))?
        .into_rgba8();

    recolor::recolor(&mut img, &palette, &opts);

    img.save(&output)
        .with_context(|| format!("failed to write output: {}", output.display()))?;

    eprintln!("done.");
    Ok(())
}

/// Build a default output path: "<stem>-<palette>.png" beside the input.
fn default_output(input: &PathBuf, variant: Variant) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wallpaper".to_string());
    let file = format!("{stem}-everforest-{}.png", variant.name());
    match input.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(file),
        _ => PathBuf::from(file),
    }
}
