use std::{isize, usize};

use bevy::{
    color::palettes::css::{BLACK, GREEN},
    log,
    pbr::wireframe::WireframeColor,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    utils::hashbrown::HashMap,
};
use bevy_egui::{egui, EguiContexts, EguiUserTextures};

use crate::util::noise::PerlinNoise;

type ChunkId = (isize, isize);

pub fn debug_show_terrain_normals(mut gizmos: Gizmos, map: Res<TerrainMap>) {
    let color = GREEN;
    for (id, chunk) in map.chunks.iter() {
        for z in 0..HEIGHT_MAP_SIZE {
            for x in 0..HEIGHT_MAP_SIZE {
                let x_scale = x as f32 / (HEIGHT_MAP_SIZE - 1) as f32;
                let xf = x_scale * CHUNK_SIZE + id.0 as f32 * CHUNK_SIZE - CHUNK_SIZE / 2.0;
                let z_scale = z as f32 / (HEIGHT_MAP_SIZE - 1) as f32;
                let zf = z_scale * CHUNK_SIZE + id.1 as f32 * CHUNK_SIZE - CHUNK_SIZE / 2.0;
                let y = chunk.height_map.get(x, z) * 2.0;
                let start = Vec3::new(xf, y, zf);
                let normal = chunk.height_map.get_normal(x, z) * 0.2;
                let end = normal + start;
                gizmos.arrow(start, end, color);
            }
        }
    }
}

#[derive(Resource)]
pub struct TerrainMap {
    chunks: HashMap<ChunkId, Chunk>,
}

#[derive(Event)]
pub struct SpawnTerrainMeshEvent(ChunkId);

const CHUNK_SIZE: f32 = 5.0;
pub struct Chunk {
    height_map: HeightMap,
}

const HEIGHT_MAP_SIZE: usize = 50;
pub struct HeightMap{
    height_data: [f32; HEIGHT_MAP_SIZE * HEIGHT_MAP_SIZE],
    normal: [Vec3; HEIGHT_MAP_SIZE * HEIGHT_MAP_SIZE]
}

fn get_index(x: usize, y: usize) -> usize {
    assert!(x < HEIGHT_MAP_SIZE);
    assert!(y < HEIGHT_MAP_SIZE);
    y + x * HEIGHT_MAP_SIZE
}

fn calculate_normal(id: ChunkId, perlin: &PerlinNoise) -> [Vec3; HEIGHT_MAP_SIZE.pow(2)]{
    let mut normals = [Vec3::ZERO; HEIGHT_MAP_SIZE.pow(2)];
    let x_offset = (HEIGHT_MAP_SIZE as isize - 1) * id.0;
    let y_offset = (HEIGHT_MAP_SIZE as isize - 1) * id.1;
    for y in 0..HEIGHT_MAP_SIZE {
        for x in 0..HEIGHT_MAP_SIZE {
            let d = 0.1;
            let xf = (x as isize + x_offset) as f32;
            let yf = (y as isize + y_offset) as f32;

            let t = perlin.fractal_brownian_motion(xf, yf+d);
            let b = perlin.fractal_brownian_motion(xf, yf-d);
            let l = perlin.fractal_brownian_motion(xf+d, yf);
            let r = perlin.fractal_brownian_motion(xf-d, yf);

            let step = 2.0* d * CHUNK_SIZE / HEIGHT_MAP_SIZE as f32;
            let x_dir = Vec3::new(step, l - r, 0.0);
            let z_dir = Vec3::new(0.0, t - b, step);
            let normal = z_dir.cross(x_dir).normalize();

            let index = get_index(x, y);
            normals[index] = normal;
        }
    }
    normals
}

impl HeightMap {
    fn new(id: ChunkId) -> Self {
        let perlin = PerlinNoise::new();
        let mut height_data = [0.0; HEIGHT_MAP_SIZE * HEIGHT_MAP_SIZE];
        for y in 0..HEIGHT_MAP_SIZE {
            for x in 0..HEIGHT_MAP_SIZE {
                let index = get_index(x, y);
                let xf = (x as isize + ((HEIGHT_MAP_SIZE as isize - 1) * id.0)) as f32;
                let yf = (y as isize + ((HEIGHT_MAP_SIZE as isize - 1) * id.1)) as f32;
                let height = perlin.fractal_brownian_motion(xf, yf);
                height_data[index] = height;
            }
        }
        let normal = calculate_normal(id, &perlin);
        Self{height_data, normal}
    }

    fn get(&self, x: usize, y: usize) -> f32 {
        let index = get_index(x, y);
        self.height_data[index]
    }
    

    fn get_normal(&self, x: usize, y: usize) -> Vec3 {
        let index = get_index(x, y);
        self.normal[index]
        // if x <= 0 || y <= 0 {
        //     return Vec3::X;
        // }
        // if x >= HEIGHT_MAP_SIZE - 1 || y >= HEIGHT_MAP_SIZE - 1 {
        //     return Vec3::X;
        // }
        // let index_t = get_index(x, y + 1);
        // let index_b = get_index(x, y - 1);
        // let index_l = get_index(x + 1, y);
        // let index_r = get_index(x - 1, y);
        // let t = self.height_data[index_t] * 2.0;
        // let b = self.height_data[index_b] * 2.0;
        // let l = self.height_data[index_l] * 2.0;
        // let r = self.height_data[index_r] * 2.0;
        // let step = 2.0 * CHUNK_SIZE / HEIGHT_MAP_SIZE as f32;
        // let x_dir = Vec3::new(step, l - r, 0.0);
        // let z_dir = Vec3::new(0.0, t - b, step);
        // z_dir.cross(x_dir).normalize()
    }
}

pub fn spawn_terrain_map(mut event: EventWriter<SpawnTerrainMeshEvent>, mut cmd: Commands) {
    let mut chunks = HashMap::new();
    for x in -1..=1 {
        for z in -1..=1 {
            let id = (x, z);
            chunks.insert(
                id,
                Chunk {
                    height_map: HeightMap::new(id),
                },
            );
            event.send(SpawnTerrainMeshEvent(id));
        }
    }

    let map = TerrainMap { chunks };
    cmd.insert_resource(map);
}

pub fn spawn_terrain_plain(
    mut cmd: Commands,
    mut event: EventReader<SpawnTerrainMeshEvent>,
    terrain_map: Res<TerrainMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in event.read() {
        log::info!("Spawn Mesh for id {:?}", ev.0);
        let Some(chunk) = terrain_map.chunks.get(&ev.0) else {
            return;
        };
        let mesh = create_terrain_mesh(&chunk.height_map);
        let mesh_handle = meshes.add(mesh);
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb_u8(125, 125, 125),
            ..default()
        });
        let x_offset = ev.0 .0 as f32 * CHUNK_SIZE;
        let z_offset = ev.0 .1 as f32 * CHUNK_SIZE;

        cmd.spawn((
            MaterialMeshBundle {
                mesh: mesh_handle,
                material,
                transform: Transform::from_xyz(x_offset, 0.0, z_offset),
                ..default()
            },
            WireframeColor {
                color: BLACK.into(),
            },
        ));
    }
}

fn create_terrain_mesh(map: &HeightMap) -> Mesh {
    let size = CHUNK_SIZE;
    let width = HEIGHT_MAP_SIZE;
    let depth = HEIGHT_MAP_SIZE;
    let height = 2.0;

    let mut vertex_positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut indices: Vec<u32> = vec![];
    for x in 0..width {
        for z in 0..depth {
            let position = [
                size / (width as f32 - 1.0) * x as f32 - size / 2.0,
                height * map.get(x, z),
                size / (depth as f32 - 1.0) * z as f32 - size / 2.0,
            ];
            vertex_positions.push(position);
        }
    }
    for x in 0..width {
        for z in 0..depth {
            // let normal = [0.0, 1.0, 0.0];
            let normal = map.get_normal(x, z);
            let normal = [normal.x, normal.y, normal.z];
            normals.push(normal);
        }
    }

    for x in 0..(width - 1) {
        for z in 0..(depth - 1) {
            let index = (z * depth + x) as u32;
            //first triangle
            indices.push(index);
            indices.push(index + 1);
            indices.push(index + depth as u32);

            //secound triangle
            indices.push(index + 1);
            indices.push(index + depth as u32 + 1);
            indices.push(index + depth as u32);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

fn create_plane(size: f32, sub_div: u32) -> Mesh {
    let perlin = PerlinNoise::new();
    let width = 2 + sub_div;
    let depth = 2 + sub_div;

    let mut vertex_positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut indices: Vec<u32> = vec![];
    for x in 0..width {
        for z in 0..depth {
            let y = perlin.fractal_brownian_motion(x as f32, z as f32);
            let position = [
                size / width as f32 * x as f32 - size / 2.0,
                y,
                // -5.0 * ((x as f32 / width as f32) - 0.5).powi(2),
                size / depth as f32 * z as f32 - size / 2.0,
            ];
            vertex_positions.push(position);
        }
    }
    for _ in 0..width {
        for _ in 0..depth {
            let normal = [0.0, 1.0, 0.0];
            normals.push(normal);
        }
    }

    for x in 0..(width - 1) {
        for z in 0..(depth - 1) {
            let index = z * depth + x;
            //first triangle
            indices.push(index);
            indices.push(index + 1);
            indices.push(index + depth);

            //secound triangle
            indices.push(index + 1);
            indices.push(index + depth + 1);
            indices.push(index + depth);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

fn noise_vec(width: u32, height: u32) -> Vec<u8> {
    let mut pixel_data = Vec::with_capacity((width * height * 4) as usize);
    let perlin = PerlinNoise::new();

    for y in 0..height {
        for x in 0..width {
            let xi = x as f32;
            let yi = y as f32;
            let p = (perlin.fractal_brownian_motion(xi, yi) + 1.0) / 2.0;
            let val = 255.0 * p;
            let a = 255;

            pixel_data.push(val as u8);
            pixel_data.push(val as u8);
            pixel_data.push(val as u8);
            pixel_data.push(a);
        }
    }
    pixel_data
}

#[derive(Deref, Resource)]
pub struct HeighMapImage(Handle<Image>);

type Pixel = [u8; 4];
type PixelData = Vec<[u8; 4]>;

pub fn pixel_data_from_height_map(map: &HeightMap) -> PixelData {
    map.height_data
        .iter()
        .map(|y| {
            let v = (((y + 1.0) / 2.0) * 255.0) as u8;
            [v, v, v, 255]
        })
        .collect()
}

pub fn pixel_data_empty(width: u32, height: u32) -> PixelData {
    (0..(width * height))
        .map(|i| {
            let v = (255.0 * (i as f32 / (width as f32 * height as f32))) as u8;
            [v, v, v, 255]
        })
        .collect()
}

pub fn create_image(width: u32, height: u32, pixels: PixelData) -> Image {
    let pixel_data = pixels.into_iter().flatten().collect();
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    )
}

pub fn update_height_map_image(
    terrain_map: Res<TerrainMap>,
    image_handle: Res<HeighMapImage>,
    mut images: ResMut<Assets<Image>>,
    mut event: EventReader<SpawnTerrainMeshEvent>,
) {
    for ev in event.read() {
        let Some(chunk) = terrain_map.chunks.get(&ev.0) else {
            break;
        };
        let map = &chunk.height_map;
        let pixels = pixel_data_from_height_map(map);
        let data = pixels.into_iter().flatten().collect();

        let Some(image) = images.get_mut(&image_handle.0) else {
            break;
        };
        image.data = data;
    }
}

pub fn setup_image(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
) {
    let data = pixel_data_empty(HEIGHT_MAP_SIZE as u32, HEIGHT_MAP_SIZE as u32);
    let image = create_image(HEIGHT_MAP_SIZE as u32, HEIGHT_MAP_SIZE as u32, data);
    let handle = images.add(image);
    egui_user_textures.add_image(handle.clone());
    cmd.insert_resource(HeighMapImage(handle));
}

pub fn debug_ui_system(mut ctx: EguiContexts, my_image: Res<HeighMapImage>) {
    let my_image_id = ctx.image_id(&my_image).unwrap();
    egui::Window::new("Terrain Controller").show(ctx.ctx_mut(), |ui| {
        ui.image(egui::load::SizedTexture::new(
            my_image_id,
            egui::vec2(100., 100.),
        ));
    });
}
