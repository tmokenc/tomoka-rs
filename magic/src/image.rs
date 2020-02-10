use crate::Result;
use image::{DynamicImage, ImageFormat, ImageOutputFormat};

pub enum ImageProcess {
    Rotate(RotateAngle),
    Flip(FlipType),
}

pub enum RotateAngle {
    Left,
    Right,
    Rotate180,
}

pub enum FlipType {
    Vertical,
    Horizontal,
}

pub type RGB = (u8, u8, u8);

/// Process an image
pub fn image_process<B: AsRef<[u8]>>(buf: B, process: ImageProcess) -> Result<Vec<u8>> {
    let (image, format) = get_image(buf.as_ref())?;
    let result = match process {
        ImageProcess::Rotate(direction) => match direction {
            RotateAngle::Left => image.rotate270(),
            RotateAngle::Right => image.rotate90(),
            RotateAngle::Rotate180 => image.rotate180(),
        },

        ImageProcess::Flip(direction) => match direction {
            FlipType::Horizontal => image.fliph(),
            FlipType::Vertical => image.flipv(),
        },
    };

    let mut data = Vec::with_capacity(buf.as_ref().len());
    let out_format = get_output_format(format);
    result.write_to(&mut data, out_format)?;

    Ok(data)
}

/// Get a buffer of image and direction of the rotation and return the buffer of a
#[inline]
pub fn rotate(buf: impl AsRef<[u8]>, direction: RotateAngle) -> Result<Vec<u8>> {
    image_process(buf, ImageProcess::Rotate(direction))
}

/// Flip the image
#[inline]
pub fn flip(buf: impl AsRef<[u8]>, direction: FlipType) -> Result<Vec<u8>> {
    image_process(buf, ImageProcess::Flip(direction))
}

/// Get dominated colors from image
pub fn get_dominanted_colors(buf: impl AsRef<[u8]>) -> Result<Vec<RGB>> {
    let image = image::load_from_memory(buf.as_ref())?;
    let has_alpha = image.color() == image::ColorType::Rgba8;

    let colors = dominant_color::get_colors(&image.to_bytes(), has_alpha)
        .into_iter()
        .map(|v| (v.r, v.g, v.b))
        .collect();

    Ok(colors)
}

fn get_image(buf: impl AsRef<[u8]>) -> Result<(DynamicImage, ImageFormat)> {
    let format = image::guess_format(buf.as_ref())?;
    let image = image::load_from_memory(buf.as_ref())?;

    Ok((image, format))
}

// get the output format, fallback to .BMP if not supported
fn get_output_format(format: ImageFormat) -> ImageOutputFormat {
    let output = ImageOutputFormat::from(format);

    match output {
        ImageOutputFormat::Unsupported(_) => ImageOutputFormat::Bmp,
        v => v,
    }
}
