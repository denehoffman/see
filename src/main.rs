use anyhow::{Result, bail};
use clap::Parser;
use see::{image::TerminalImage, render::FitMode};
use std::{
    io::{self, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(name = "see")]
#[command(version)]
#[command(about = "View images in the terminal using real graphics protocols")]
struct Cli {
    /// Image file to display.
    image: PathBuf,

    /// Resize to this pixel width. Height is computed from aspect ratio.
    /// If used with --height, fits inside the provided box.
    #[arg(short = 'W', long)]
    width: Option<u32>,

    /// Resize to this pixel height. Width is computed from aspect ratio.
    /// If used with --width, fits inside the provided box.
    #[arg(short = 'H', long)]
    height: Option<u32>,

    /// Fit to terminal width only. Height may exceed the visible window.
    #[arg(short = 'f', long)]
    full_width: bool,

    /// Do not resize the image.
    #[arg(short = 'o', long)]
    original: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let options = resolve_fit_mode(&cli)?;
    let img = TerminalImage::open(&cli.image)?;

    let mut stdout = io::stdout().lock();
    see::render::render(&mut stdout, &img, &options)?;
    stdout.flush()?;

    Ok(())
}

fn resolve_fit_mode(cli: &Cli) -> Result<FitMode> {
    if cli.original {
        if cli.width.is_some() || cli.height.is_some() || cli.full_width {
            bail!("--original cannot be used with --width, --height, or --full-width");
        }

        return Ok(FitMode::Original);
    }

    if cli.full_width {
        if cli.width.is_some() || cli.height.is_some() {
            bail!("--full-width cannot be used with --width or --height");
        }

        return Ok(FitMode::TerminalWidth);
    }

    match (cli.width, cli.height) {
        (Some(width), Some(height)) => Ok(FitMode::Pixels { width, height }),
        (Some(width), None) => Ok(FitMode::WidthPixels(width)),
        (None, Some(height)) => Ok(FitMode::HeightPixels(height)),
        (None, None) => Ok(FitMode::Terminal),
    }
}
