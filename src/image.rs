use anyhow::{Result, anyhow, bail};
use image::{
    ColorType, ImageEncoder, RgbaImage,
    codecs::png::PngEncoder,
    imageops::{self, FilterType},
};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct TerminalImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl TerminalImage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path)?;
        Ok(Self::from_dynamic(img))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_dynamic(img))
    }

    pub fn from_dynamic(img: image::DynamicImage) -> Self {
        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        Self {
            width,
            height,
            rgba: rgba.into_raw(),
        }
    }

    pub fn to_png_bytes(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        PngEncoder::new(&mut out).write_image(
            &self.rgba,
            self.width,
            self.height,
            ColorType::Rgba8.into(),
        )?;
        Ok(out)
    }

    pub fn resized_to_fit(&self, max_width: u32, max_height: u32) -> Result<Self> {
        if self.width == 0 || self.height == 0 {
            bail!(
                "image has invalid dimensions: {}x{}",
                self.width,
                self.height
            );
        }
        if max_width == 0 || max_height == 0 {
            bail!(
                "target dimensions are invalid: {}x{}",
                max_width,
                max_height
            );
        }

        let scale_x = max_width as f64 / self.width as f64;
        let scale_y = max_height as f64 / self.height as f64;
        let scale = scale_x.min(scale_y);
        let new_width = ((self.width as f64 * scale).floor() as u32)
            .max(1)
            .min(max_width);
        let new_height = ((self.height as f64 * scale).floor() as u32)
            .max(1)
            .min(max_height);

        if new_width == self.width && new_height == self.height {
            return Ok(self.clone());
        }

        let rgba = RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| anyhow!("invalid RGBA buffer"))?;

        let resized = imageops::resize(&rgba, new_width, new_height, FilterType::Lanczos3);
        Ok(Self {
            width: new_width,
            height: new_height,
            rgba: resized.into_raw(),
        })
    }

    pub fn resized_to_width(&self, target_width: u32) -> Result<Self> {
        if self.width == 0 || self.height == 0 {
            bail!(
                "image has invalid dimensions: {}x{}",
                self.width,
                self.height
            );
        }

        if target_width == 0 {
            bail!("target width must be non-zero");
        }

        let scale = target_width as f64 / self.width as f64;

        let new_width = target_width;
        let new_height = ((self.height as f64 * scale).floor() as u32).max(1);

        if new_width == self.width && new_height == self.height {
            return Ok(self.clone());
        }

        let rgba = RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| anyhow!("invalid RGBA buffer"))?;

        let resized = imageops::resize(&rgba, new_width, new_height, FilterType::Lanczos3);

        Ok(Self {
            width: new_width,
            height: new_height,
            rgba: resized.into_raw(),
        })
    }

    pub fn resized_to_height(&self, target_height: u32) -> Result<Self> {
        if self.width == 0 || self.height == 0 {
            bail!(
                "image has invalid dimensions: {}x{}",
                self.width,
                self.height
            );
        }

        if target_height == 0 {
            bail!("target height must be non-zero");
        }

        let scale = target_height as f64 / self.height as f64;

        let new_width = ((self.width as f64 * scale).floor() as u32).max(1);
        let new_height = target_height;

        if new_width == self.width && new_height == self.height {
            return Ok(self.clone());
        }

        let rgba = RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| anyhow!("invalid RGBA buffer"))?;

        let resized = imageops::resize(&rgba, new_width, new_height, FilterType::Lanczos3);

        Ok(Self {
            width: new_width,
            height: new_height,
            rgba: resized.into_raw(),
        })
    }
}
