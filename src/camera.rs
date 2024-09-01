use bevy::prelude::*;

pub struct OrbitCameraPlugin;
pub struct CameraConfig {}

#[derive(Component)]
pub struct CameraController {
    y_pos: f32,
    y_look_pos: f32,
    rot_pos: f32,
    auto_rot: bool,
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (update_camera, move_camera));
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {}
    }
}

pub fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(5., 5., 5.).looking_at(Vec3::new(0., 2., 0.), Vec3::Y),
            ..Default::default()
        },
        CameraController {
            y_pos: 5.0,
            y_look_pos: 2.0,
            rot_pos: 0.0,
            auto_rot: false,
        },
    ));
}

// Camera
pub fn move_camera(mut q_camera: Query<(&mut Transform, &CameraController)>) {
    let radius = 8.0;
    let (mut trans, cam) = q_camera.single_mut();
    trans.translation.y = cam.y_pos;
    trans.translation.x = f32::cos(cam.rot_pos) * radius;
    trans.translation.z = f32::sin(cam.rot_pos) * radius;
    *trans = trans.looking_at(Vec3::new(0.0, cam.y_look_pos, 0.0), Vec3::Y);
}

pub fn update_camera(
    key_input: Res<ButtonInput<KeyCode>>,
    mut q_camera: Query<&mut CameraController>,
    time: Res<Time>,
) {
    let speed = 1.0;
    let rot_speed = 1.0;
    let mut cam = q_camera.single_mut();
    let mut cam_move = 0.0;
    if key_input.pressed(KeyCode::ArrowUp) {
        cam_move += 1.0
    }
    if key_input.pressed(KeyCode::ArrowDown) {
        cam_move += -1.0;
    }

    let mut cam_rotate = 0.0;
    if key_input.pressed(KeyCode::ArrowRight) {
        cam_rotate += 1.0
    }
    if key_input.pressed(KeyCode::ArrowLeft) {
        cam_rotate += -1.0;
    }
    if key_input.just_pressed(KeyCode::KeyM) {
        cam.auto_rot = !cam.auto_rot;
    }
    // auto rotation
    if cam.auto_rot && cam_rotate == 0.0 {
        cam_rotate = 0.5;
    }
    cam.y_pos += speed * cam_move * time.delta_seconds();
    cam.y_look_pos = cam.y_pos - 2.0;
    cam.rot_pos += rot_speed * cam_rotate * time.delta_seconds();
}
