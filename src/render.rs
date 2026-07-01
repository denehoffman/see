use std::io::Write;

use anyhow::Result;

use crate::{image::TerminalImage, protocol::Protocol};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum FitMode {
    /// Fit to full terminal window.
    #[default]
    Terminal,
    /// Fit to terminal width only. Height may exceed the visible window.
    TerminalWidth,
    /// Resize to this pixel width. Height is computed from aspect ratio.
    WidthPixels(u32),
    /// Resize to this pixel height. Width is computed from aspect ratio.
    HeightPixels(u32),
    /// Fit inside bounding box.
    Pixels { width: u32, height: u32 },
    /// No resizing
    Original,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedRenderOptions {
    pub width_pixels: u32,
    pub height_pixels: u32,
    pub width_cells: Option<u16>,
    pub height_cells: Option<u16>,
}

#[derive(Clone, Copy, Debug)]
struct TerminalGeometry {
    columns: u16,
    rows: u16,
    width_pixels: u32,
    height_pixels: u32,
    cell_width_pixels: u32,
    cell_height_pixels: u32,
}

pub fn render<W: Write>(out: &mut W, img: &TerminalImage, options: &FitMode) -> Result<()> {
    let protocol = Protocol::detect()
        .ok_or_else(|| anyhow::anyhow!("unsupported terminal graphics protocol"))?;

    let (img, resolved) = resolve_image(img, options)?;
    protocol.render(out, &img, &resolved)
}

fn resolve_image(
    img: &TerminalImage,
    options: &FitMode,
) -> Result<(TerminalImage, ResolvedRenderOptions)> {
    match options {
        FitMode::Original => {
            let img = img.clone();

            Ok((
                img.clone(),
                ResolvedRenderOptions {
                    width_pixels: img.width,
                    height_pixels: img.height,
                    width_cells: None,
                    height_cells: None,
                },
            ))
        }

        FitMode::WidthPixels(width) => {
            let img = img.resized_to_width(*width)?;

            Ok((
                img.clone(),
                ResolvedRenderOptions {
                    width_pixels: img.width,
                    height_pixels: img.height,
                    width_cells: None,
                    height_cells: None,
                },
            ))
        }

        FitMode::HeightPixels(height) => {
            let img = img.resized_to_height(*height)?;

            Ok((
                img.clone(),
                ResolvedRenderOptions {
                    width_pixels: img.width,
                    height_pixels: img.height,
                    width_cells: None,
                    height_cells: None,
                },
            ))
        }

        FitMode::Pixels { width, height } => {
            let img = img.resized_to_fit(*width, *height)?;

            Ok((
                img.clone(),
                ResolvedRenderOptions {
                    width_pixels: img.width,
                    height_pixels: img.height,
                    width_cells: None,
                    height_cells: None,
                },
            ))
        }

        FitMode::Terminal => {
            let terminal = TerminalGeometry::current()?;
            let img = img.resized_to_fit(terminal.width_pixels, terminal.height_pixels)?;

            let width_cells = div_ceil(img.width, terminal.cell_width_pixels)
                .min(terminal.columns as u32)
                .max(1) as u16;

            let height_cells = div_ceil(img.height, terminal.cell_height_pixels)
                .min(terminal.rows as u32)
                .max(1) as u16;

            Ok((
                img.clone(),
                ResolvedRenderOptions {
                    width_pixels: img.width,
                    height_pixels: img.height,
                    width_cells: Some(width_cells),
                    height_cells: Some(height_cells),
                },
            ))
        }
        FitMode::TerminalWidth => {
            let terminal = TerminalGeometry::current()?;

            let img = img.resized_to_width(terminal.width_pixels)?;

            let height_cells = div_ceil(img.height, terminal.cell_height_pixels).max(1) as u16;

            Ok((
                img.clone(),
                ResolvedRenderOptions {
                    width_pixels: img.width,
                    height_pixels: img.height,
                    width_cells: Some(terminal.columns),
                    height_cells: Some(height_cells),
                },
            ))
        }
    }
}

impl TerminalGeometry {
    fn current() -> Result<Self> {
        let size = crossterm::terminal::window_size()?;

        let columns = size.columns.max(1);
        let rows = size.rows.max(1);

        let width_pixels = if size.width > 0 {
            size.width as u32
        } else {
            columns as u32 * 8
        };

        let height_pixels = if size.height > 0 {
            size.height as u32
        } else {
            rows as u32 * 16
        };

        let cell_width_pixels = (width_pixels / columns as u32).max(1);
        let cell_height_pixels = (height_pixels / rows as u32).max(1);

        Ok(Self {
            columns,
            rows,
            width_pixels,
            height_pixels,
            cell_width_pixels,
            cell_height_pixels,
        })
    }
}

fn div_ceil(lhs: u32, rhs: u32) -> u32 {
    lhs.saturating_add(rhs.saturating_sub(1)) / rhs.max(1)
}
