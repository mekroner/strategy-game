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
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiUserTextures};
use ground::{debug_ui_system, setup_image, SpawnTerrainMeshEvent};

mod camera;
mod ground;
mod util;

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
        .add_plugins(camera::OrbitCameraPlugin)
        .add_systems(
            Startup,
            (spawn_light, setup_image, ground::spawn_terrain_map),
        )
        .add_systems(
            Update,
            (
                ground::spawn_terrain_plain, 
                ground::update_height_map_image
            ),
        )
        .add_systems(Update, (debug_ui_system, 
            // ground::debug_show_terrain_normals
        ))
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
