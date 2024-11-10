use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::Image,
};

use super::{HeightMap, HEIGHT_MAP_SIZE};

pub type Pixel = [u8; 4];

pub struct PixelData {
    pixels: Vec<Pixel>,
    width: u32,
    height: u32,
}

impl PixelData {
    pub fn from_height_map(map: &HeightMap) -> Self {
        let pixels = map
            .height_data
            .iter()
            .map(|y| {
                let v = (((y + 1.0) / 2.0) * 255.0) as u8;
                [v, v, v, 255]
            })
            .collect();
        Self {
            pixels,
            width: HEIGHT_MAP_SIZE as u32,
            height: HEIGHT_MAP_SIZE as u32,
        }
    }

    pub fn empty(width: u32, height: u32) -> Self {
        let pixels = (0..(width * height))
            .map(|i| {
                let v = (255.0 * (i as f32 / (width as f32 * height as f32))) as u8;
                [v, v, v, 255]
            })
            .collect();
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn splat(width: u32, height: u32, pixel: Pixel) -> Self {
        let pixels = vec![pixel; width as usize * height as usize];
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn flat_data(&self) -> Vec<u8> {
        self.pixels.clone().into_iter().flatten().collect()
    }

    pub fn to_image(&self) -> Image {
        let pixel_data = self.flat_data();
        Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixel_data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        )
    }
}
