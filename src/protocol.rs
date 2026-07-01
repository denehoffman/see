use crate::{image::TerminalImage, render::ResolvedRenderOptions};
use anyhow::Result;
use base64::Engine;
use icy_sixel::SixelImage;
use std::io::prelude::Write;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Protocol {
    Kitty,
    Iterm2,
    Sixel,
}
impl Protocol {
    pub(crate) fn render<W: Write>(
        &self,
        out: &mut W,
        img: &TerminalImage,
        options: &ResolvedRenderOptions,
    ) -> Result<()> {
        match self {
            Protocol::Kitty => render_kitty(out, img, options),
            Protocol::Iterm2 => render_iterm2(out, img, options),
            Protocol::Sixel => render_sixel(out, img, options),
        }
    }

    pub(crate) fn detect() -> Option<Self> {
        let term = env_lower("TERM");
        let term_program = env_lower("TERM_PROGRAM");
        let lc_terminal = env_lower("LC_TERMINAL");
        let vte_version = std::env::var_os("VTE_VERSION").is_some();

        // Kitty protocol terminals.
        if has_env("KITTY_WINDOW_ID") || term.contains("xterm-kitty") {
            return Some(Self::Kitty);
        }

        if term.contains("xterm-ghostty")
            || term_program.contains("ghostty")
            || has_env("GHOSTTY_RESOURCES_DIR")
        {
            return Some(Self::Kitty);
        }

        if term_program.contains("wezterm") || has_env("WEZTERM_PANE") || term.contains("wezterm") {
            return Some(Self::Kitty);
        }

        // iTerm2 protocol terminals.
        if term_program == "iterm.app"
            || term_program.contains("iterm")
            || lc_terminal.contains("iterm2")
        {
            return Some(Self::Iterm2);
        }

        // Sixel terminals. This is conservative and env-based.
        if term.contains("sixel")
            || term.contains("foot")
            || term.contains("mlterm")
            || term.contains("xterm-direct")
            || term_program.contains("rio")
            || term_program.contains("contour")
            || term_program.contains("konsole")
            || has_env("KONSOLE_VERSION")
            || has_env("WT_SESSION")
        {
            return Some(Self::Sixel);
        }

        // Many VTE terminals can be built with SIXEL, but not all of them.
        // Only use this as a last-resort hint.
        if vte_version && term.contains("sixel") {
            return Some(Self::Sixel);
        }

        None
    }
}

fn render_kitty<W: Write>(
    out: &mut W,
    img: &TerminalImage,
    options: &ResolvedRenderOptions,
) -> Result<()> {
    let mut control = format!("a=T,f=32,t=d,s={},v={}", img.width, img.height);

    if let Some(cols) = options.width_cells {
        control.push_str(&format!(",c={cols}"));
    }

    if let Some(rows) = options.height_cells {
        control.push_str(&format!(",r={rows}"));
    }

    let encoded = base64::engine::general_purpose::STANDARD.encode(&img.rgba);
    let mut chunks = encoded.as_bytes().chunks(4096).peekable();

    if let Some(first) = chunks.next() {
        let more = if chunks.peek().is_some() { 1 } else { 0 };

        write!(out, "\x1b_G{control},m={more};")?;
        out.write_all(first)?;
        write!(out, "\x1b\\")?;
    }

    while let Some(chunk) = chunks.next() {
        let more = if chunks.peek().is_some() { 1 } else { 0 };

        write!(out, "\x1b_Gm={more};")?;
        out.write_all(chunk)?;
        write!(out, "\x1b\\")?;
    }

    Ok(())
}

fn render_iterm2<W: Write>(
    out: &mut W,
    img: &TerminalImage,
    options: &ResolvedRenderOptions,
) -> Result<()> {
    let png = img.to_png_bytes()?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&png);
    write!(
        out,
        "\x1b]1337;File=inline=1;width={}px;height={}px;preserveAspectRatio=1:{}\x07",
        options.width_pixels, options.height_pixels, encoded,
    )?;
    Ok(())
}

fn render_sixel<W: Write>(
    out: &mut W,
    img: &TerminalImage,
    _options: &ResolvedRenderOptions,
) -> Result<()> {
    let sixel_img =
        SixelImage::from_rgba(img.rgba.clone(), img.width as usize, img.height as usize);

    let escape = sixel_img.encode()?;
    write!(out, "{escape}")?;

    Ok(())
}

fn env_lower(key: &str) -> String {
    std::env::var(key).unwrap_or_default().to_ascii_lowercase()
}

fn has_env(key: &str) -> bool {
    std::env::var_os(key).is_some()
}
