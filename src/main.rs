use anyhow::{Result, anyhow, bail};
use pico_args::Arguments;
use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::{
    image::TerminalImage,
    render::{FitMode, render},
};

mod image;
mod protocol;
mod render;

const HELP: &str = "\
see

View images in the terminal using real graphics protocols.

USAGE:
    see [OPTIONS] <IMAGE>...

OPTIONS:
    -f, --full-width       Fit to terminal width; height may exceed the visible window
    -o, --original         Display at original image size
    -W, --width <PX>       Resize to this pixel width; height is computed automatically
    -H, --height <PX>      Resize to this pixel height; width is computed automatically
    -h, --help             Print help
    -V, --version          Print version

EXAMPLES:
    see image.png
    see a.png b.png c.jpg
    see *.png
    see image.png -f
    see image.png -o
    see image.png -W 800
    see image.png -H 600
    see image.png -W 800 -H 600
";

fn main() -> Result<()> {
    let cli = Cli::parse()?;

    if cli.help {
        print!("{HELP}");
        return Ok(());
    }

    if cli.version {
        println!("see {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let options = resolve_fit_mode(&cli)?;
    if cli.images.is_empty() {
        return Err(anyhow!("missing image path\n\n{HELP}"));
    }

    let mut stdout = io::stdout().lock();

    for image in &cli.images {
        let img = TerminalImage::open(image)?;
        render(&mut stdout, &img, &options)?;
        writeln!(stdout)?;
        stdout.flush()?;
    }

    Ok(())
}

#[derive(Debug)]
struct Cli {
    images: Vec<PathBuf>,
    width: Option<u32>,
    height: Option<u32>,
    original: bool,
    full_width: bool,
    help: bool,
    version: bool,
}

impl Cli {
    fn parse() -> Result<Self> {
        let mut args = Arguments::from_env();

        let help = args.contains(["-h", "--help"]);
        let version = args.contains(["-V", "--version"]);
        let original = args.contains(["-o", "--original"]);
        let full_width = args.contains(["-f", "--full-width"]);

        let width = args.opt_value_from_str(["-W", "--width"])?;
        let height = args.opt_value_from_str(["-H", "--height"])?;

        let remaining = args.finish();

        if let Some(arg) = remaining
            .iter()
            .find(|arg| arg.to_string_lossy().starts_with('-'))
        {
            bail!("unexpected argument: {}", arg.to_string_lossy());
        }

        let images = remaining.into_iter().map(PathBuf::from).collect();

        Ok(Self {
            images,
            width,
            height,
            original,
            full_width,
            help,
            version,
        })
    }
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
