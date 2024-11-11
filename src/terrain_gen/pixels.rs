use bevy::{
    log,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
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

    pub fn quantize(&mut self, steps: f32) {
        let step_size = 255.0 / (steps - 1.0);
        for pixel in self.pixels.iter_mut() {
            pixel[0] = ((pixel[0] as f32 / step_size).round() * step_size) as u8;
            pixel[1] = ((pixel[1] as f32 / step_size).round() * step_size) as u8;
            pixel[2] = ((pixel[2] as f32 / step_size).round() * step_size) as u8;
        }
    }

    pub fn apply_gradient(&mut self) {
        let gradient = vec![
            (0.1, [0, 0, 120, 255]),
            (0.3, [0, 50, 200, 255]),
            (0.4, [0, 200, 200, 255]),
            (0.45, [150, 200, 50, 255]),
            (0.5, [20, 200, 50, 255]),
            (0.7, [100, 100, 100, 255]),
            (0.9, [150, 200, 200, 255]),
        ];
        'out: for pixel in self.pixels.iter_mut() {
            let t = pixel[0] as f32 / 255.0;
            if t < gradient.first().unwrap().0 {
                *pixel = gradient.first().unwrap().1;
                continue;
            }

            if t > gradient.last().unwrap().0 {
                *pixel = gradient.last().unwrap().1;
                continue;
            }

            for i in 1..gradient.len() {
                if t < gradient[i].0 {
                    let ratio = (t - gradient[i - 1].0) / (gradient[i].0 - gradient[i - 1].0);
                    *pixel = lerp_pixel(gradient[i - 1].1, gradient[i].1, ratio);
                    continue 'out;
                }
            }

            *pixel = [255, 0, 0, 255];
        }
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

fn lerp_pixel(start: Pixel, end: Pixel, t: f32) -> Pixel {
    let r = (start[0] as f32 + t * (end[0] as f32 - start[0] as f32)) as u8;
    let g = (start[1] as f32 + t * (end[1] as f32 - start[1] as f32)) as u8;
    let b = (start[2] as f32 + t * (end[2] as f32 - start[2] as f32)) as u8;
    let a = (start[3] as f32 + t * (end[3] as f32 - start[3] as f32)) as u8;
    [r, g, b, a]
}
