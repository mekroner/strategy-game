use std::isize;
use std::ops::Deref;

use bevy::color::palettes::css::{DARK_GREY, RED};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log;
use bevy::{color::palettes::css::GREEN, prelude::*};
use strategy_game::terrain_gen::pixels::PixelData;
use strategy_game::terrain_gen::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_event::<SpawnTerrainMeshEvent>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_blank_texture)
        .add_systems(Update, gizmo_grid)
        // terrain systems
        .add_systems(Startup, spawn_terrain_map)
        .add_systems(Update, spawn_chunk_image)
        .add_systems(Startup, spawn_chunk_loader)
        .add_systems(Update, (gizmo_chunk_loader, move_chunk_loader))
        .run();
}

fn spawn_camera(mut cmd: Commands) {
    cmd.spawn(Camera2dBundle::default());
}

fn spawn_blank_texture(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let width = 50;
    let height = 50;
    let pixel_data = PixelData::splat(width, height, [255, 0, 0, 255]);
    let image = pixel_data.to_image();
    let texture_handle = textures.add(image);

    // You can now use this texture handle with materials or sprite rendering
    commands.spawn(SpriteBundle {
        texture: texture_handle,
        ..Default::default()
    });
}

pub fn spawn_chunk_image(
    mut cmd: Commands,
    mut event: EventReader<SpawnTerrainMeshEvent>,
    terrain_map: Res<TerrainMap>,
    mut textures: ResMut<Assets<Image>>,
) {
    let size = 50;
    for ev in event.read() {
        let x_offset = ev.0 * size as isize;
        let z_offset = ev.1 * size as isize;
        let pos = Vec3::new(z_offset as f32, -x_offset as f32, 0.0);

        let Some(chunk) = terrain_map.chunks.get(ev.deref()) else {
            return;
        };
        let pixel_data = PixelData::from_height_map(&chunk.height_map);
        let image = pixel_data.to_image();
        let texture_handle = textures.add(image.clone());

        // You can now use this texture handle with materials or sprite rendering
        cmd.spawn(SpriteBundle {
            texture: texture_handle,
            transform: Transform::default().with_translation(pos),
            ..Default::default()
        });
        log::info!("Spawned image {:?}.", ev);
    }
}

fn gizmo_grid(mut gizmos: Gizmos, q_loader: Query<&mut ChunkLoader>) {
    let loader = q_loader.single();
    for x in -10..=10 {
        for y in -10..=10 {
            let size = 50.0;
            let position = Vec2::new(size * x as f32, size * y as f32);
            let active_color = GREEN;
            let unactive_color = DARK_GREY;
            let mut color = unactive_color;
            // if (loader.x - x as f32).abs() < loader.range as f32
            //     && (loader.y - y as f32).abs() < loader.range as f32
            // {
            //     color = active_color
            // }
            if (loader.x - x as f32).powi(2) + (loader.y - y as f32).powi(2)
                < (loader.range as f32).powi(2)
            {
                color = active_color
            }
            gizmos.rect_2d(position, 0., Vec2::splat(size - 5.0), color);
        }
    }
}

fn spawn_chunk_loader(mut cmd: Commands) {
    cmd.spawn(ChunkLoader {
        x: 0.0,
        y: 0.0,
        range: 2,
    });
}

#[derive(Component)]
struct ChunkLoader {
    x: f32,
    y: f32,
    range: u32,
}

fn gizmo_chunk_loader(mut gizmos: Gizmos, q_loader: Query<&mut ChunkLoader>) {
    let size = 50.0;
    let loader = q_loader.single();
    let position = Vec2::new(size * loader.x, size * loader.y);
    gizmos.circle_2d(position, 10.0, RED);
}

fn move_chunk_loader(
    key_input: Res<ButtonInput<KeyCode>>,
    mut q_loader: Query<&mut ChunkLoader>,
    time: Res<Time>,
) {
    let speed = 1.0;
    let mut loader = q_loader.single_mut();
    let mut loader_move = Vec2::ZERO;
    if key_input.pressed(KeyCode::ArrowUp) {
        loader_move.y += 1.0
    }
    if key_input.pressed(KeyCode::ArrowDown) {
        loader_move.y += -1.0;
    }
    if key_input.pressed(KeyCode::ArrowRight) {
        loader_move.x += 1.0
    }
    if key_input.pressed(KeyCode::ArrowLeft) {
        loader_move.x += -1.0;
    }
    // auto rotation
    loader.x += speed * loader_move.x * time.delta_seconds();
    loader.y += speed * loader_move.y * time.delta_seconds();
}
