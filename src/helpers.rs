use fast_image_resize::{FilterType, PixelType, Resizer};
use image::RgbaImage;
use std::num::NonZeroU32;

use crate::{error::WlrsError, CropMode, Image, WallpaperData};

pub(crate) fn resize(wallpaper_data: &WallpaperData, size: [i32; 2]) -> Result<Vec<u8>, WlrsError> {
    let img = match wallpaper_data.crop_mode {
        CropMode::Fit(color) => {
            let color = color.unwrap_or([0, 0, 0]);
            resize_image(&wallpaper_data.image_data, size.map(|x| x as u32), color)?
        }
        CropMode::No(color) => {
            let color = color.unwrap_or([0, 0, 0]);
            pad(&wallpaper_data.image_data, size.map(|x| x as u32), color)?
        }
        CropMode::Crop => crop_image(&wallpaper_data.image_data, size.map(|x| x as u32))?,
    };

    Ok(img)
}

pub(crate) fn resize_image(
    image_data: &Image,
    size: [u32; 2],
    color: [u8; 3],
) -> Result<Vec<u8>, WlrsError> {
    if image_data.width() == size[0] && image_data.height() == size[1] {
        return pad(image_data, size, color);
    }

    let ratio = size[0] as f32 / size[1] as f32;
    let img_ratio = image_data.width() as f32 / image_data.height() as f32;

    let (trg_width, trg_height) = if ratio > img_ratio {
        let scale = size[1] as f32 / image_data.height() as f32;
        ((image_data.width() as f32 * scale) as u32, size[0])
    } else {
        let scale = size[0] as f32 / image_data.width() as f32;
        (size[0], (image_data.height() as f32 * scale) as u32)
    };

    let trg_w = trg_width.min(size[0]);
    let trg_h = trg_height.min(size[1]);

    let src = fast_image_resize::Image::from_vec_u8(
        image_data.width,
        image_data.height,
        image_data.data.to_vec(),
        PixelType::U8x4,
    )?;

    let new_w =
        NonZeroU32::new(trg_w).ok_or(WlrsError::SizeError("Your monitor has 0 width (???)"))?;
    let new_h =
        NonZeroU32::new(trg_h).ok_or(WlrsError::SizeError("Your monitor has 0 height (???)"))?;

    let mut dst = fast_image_resize::Image::new(new_w, new_h, PixelType::U8x4);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));

    resizer.resize(&src.view(), &mut dst_view)?;

    pad(
        &Image::new(&dst.into_vec(), new_w, new_h).unwrap(),
        size,
        color,
    )
}

pub(crate) fn pad(img: &Image, size: [u32; 2], color: [u8; 3]) -> Result<Vec<u8>, WlrsError> {
    if img.dimensions() == (size[0], size[1]) {
        return Ok(img.data.to_vec());
    }
    let color = [color[0], color[1], color[2], 255];
    let (trg_w, trg_h) = (size[0] as usize, size[1] as usize);
    let mut padded = Vec::with_capacity(trg_w * trg_h * 4);

    let img = RgbaImage::from_raw(img.width(), img.height(), img.data.to_vec()).unwrap();
    let img = image::imageops::crop_imm(&img, 0, 0, trg_w as u32, trg_h as u32).to_image();
    let (img_w, img_h) = img.dimensions();
    let (img_w, img_h) = (img_w as usize, img_h as usize);
    let raw_img = img.into_vec();

    (0..(((trg_h - img_h) / 2) * trg_w)).for_each(|_| {
        padded.push(color[3]);
        padded.push(color[2]);
        padded.push(color[1]);
        padded.push(color[0]);
    });

    let left_border_w = (trg_w - img_w) / 2;
    let right_border_w = left_border_w + (img_w % 2);

    (0..img_h).for_each(|row| {
        (0..left_border_w).for_each(|_| {
            padded.extend_from_slice(&color);
        });

        raw_img[(row * img_w * 4)..((row + 1) * img_w * 4)]
            .chunks_exact(4)
            .for_each(|pixel| {
                padded.push(color[3]);
                padded.push(pixel[2]);
                padded.push(pixel[1]);
                padded.push(pixel[0]);
            });

        (0..right_border_w).for_each(|_| {
            padded.extend_from_slice(&color);
        });
    });

    while padded.len() < (trg_w * trg_h * 4) {
        padded.extend_from_slice(&color);
    }

    Ok(padded)
}

pub fn crop_image(img: &Image, size: [u32; 2]) -> Result<Vec<u8>, WlrsError> {
    if img.dimensions() == size.into() {
        return Ok(img.data.to_vec());
    }

    let pixel_type = PixelType::U8x4;
    let src = fast_image_resize::Image::from_vec_u8(
        img.width,
        img.height,
        img.data.to_vec(),
        pixel_type,
    )?;

    let new_w = NonZeroU32::new(size[0]).ok_or(WlrsError::SizeError("Empty image"))?;
    let new_h = NonZeroU32::new(size[1]).ok_or(WlrsError::SizeError("Empty image"))?;
    let mut src_view = src.view();
    src_view.set_crop_box_to_fit_dst_size(new_w, new_h, Some((0.5, 0.5)));

    let mut dst = fast_image_resize::Image::new(new_w, new_h, pixel_type);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));
    resizer.resize(&src_view, &mut dst_view)?;

    Ok(dst.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_image() {
        let img = Image::new(
            &[0; 1920 * 1080 * 4],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        )
        .unwrap();
        let resized = resize_image(&img, img.dimensions().into(), [0, 0, 0]).unwrap();
        assert_eq!(
            resized.len(),
            img.width() as usize * img.height() as usize * 4
        );

        let img = Image::new(
            &[0; 500 * 1080 * 4],
            NonZeroU32::new(500).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        )
        .unwrap();
        let resized = resize_image(&img, [1920, img.height()], [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), 1920 * img.height() as usize * 4);

        let img = Image::new(
            &[0; 1920 * 500 * 4],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(500).unwrap(),
        )
        .unwrap();
        let resized = resize_image(&img, [img.width(), 1080], [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), img.width() as usize * 1080 * 4);

        let img = Image::new(
            &[0; 3840 * 2160 * 4],
            NonZeroU32::new(3840).unwrap(),
            NonZeroU32::new(2160).unwrap(),
        )
        .unwrap();
        let resized = resize_image(&img, [1920, 1080], [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), 1920 * 1080 * 4);
    }

    #[test]
    fn test_pad() {
        let img = Image::new(
            &[0; 1920 * 1080 * 4],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        )
        .unwrap();
        let padded = pad(&img, img.dimensions().into(), [0, 0, 0]).unwrap();
        assert_eq!(
            padded.len(),
            img.width() as usize * img.height() as usize * 4
        );

        let img = Image::new(
            &[0; 500 * 1080 * 4],
            NonZeroU32::new(500).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        )
        .unwrap();
        let padded = pad(&img, [1920, img.height()], [0, 0, 0]).unwrap();
        assert_eq!(padded.len(), 1920 * img.height() as usize * 4);

        let img = Image::new(
            &[0; 1920 * 500 * 4],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(500).unwrap(),
        )
        .unwrap();
        let padded = pad(&img, [img.width(), 1080], [0, 0, 0]).unwrap();
        assert_eq!(padded.len(), img.width() as usize * 1080 * 4);

        let img = Image::new(
            &[0; 3840 * 2160 * 4],
            NonZeroU32::new(3840).unwrap(),
            NonZeroU32::new(2160).unwrap(),
        )
        .unwrap();
        let resized = pad(&img, [1920, 1080], [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), 1920 * 1080 * 4);
    }

    #[test]
    fn test_crop_image() {
        let img = Image::new(
            &[0; 1920 * 1080 * 4],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        )
        .unwrap();
        let cropped = crop_image(&img, img.dimensions().into()).unwrap();
        assert_eq!(
            cropped.len(),
            img.width() as usize * img.height() as usize * 4
        );

        let img = Image::new(
            &[0; 500 * 1080 * 4],
            NonZeroU32::new(500).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        )
        .unwrap();
        let cropped = crop_image(&img, [1920, img.height()]).unwrap();
        assert_eq!(cropped.len(), 1920 * img.height() as usize * 4);

        let img = Image::new(
            &[0; 1920 * 500 * 4],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(500).unwrap(),
        )
        .unwrap();
        let cropped = crop_image(&img, [img.width(), 1080]).unwrap();
        assert_eq!(cropped.len(), img.width() as usize * 1080 * 4);

        let img = Image::new(
            &[0; 3840 * 2160 * 4],
            NonZeroU32::new(3840).unwrap(),
            NonZeroU32::new(2160).unwrap(),
        )
        .unwrap();
        let resized = crop_image(&img, [1920, 1080]).unwrap();
        assert_eq!(resized.len(), 1920 * 1080 * 4);
    }
}
