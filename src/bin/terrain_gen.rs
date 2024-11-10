use std::f32::consts::PI;

use bevy::{
    color::palettes::css::WHITE,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_egui::EguiPlugin;
use strategy_game::camera::OrbitCameraPlugin;
use strategy_game::terrain_gen;
use strategy_game::terrain_gen::{debug_ui_system, setup_image, SpawnTerrainMeshEvent};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            WireframePlugin,
            EguiPlugin,
        ))
        .add_event::<SpawnTerrainMeshEvent>()
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .add_plugins(OrbitCameraPlugin)
        .add_systems(
            Startup,
            (spawn_light, setup_image, terrain_gen::spawn_terrain_map),
        )
        .add_systems(
            Update,
            (
                terrain_gen::spawn_terrain_plain,
                terrain_gen::update_height_map_image,
            ),
        )
        .add_systems(
            Update,
            (debug_ui_system, terrain_gen::debug_show_terrain_normals),
        )
        .run();
}

fn spawn_light(mut cmd: Commands) {
    // cmd.spawn(PointLightBundle {
    //     transform: Transform::from_xyz(0.0, 4.0, 0.0),
    //     ..default()
    // });
    cmd.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..Default::default()
    });
}
