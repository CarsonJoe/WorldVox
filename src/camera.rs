// src/camera.rs
use bevy::{
    prelude::*,
    input::mouse::MouseMotion,
    window::CursorGrabMode,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (
                camera_controller,
                toggle_cursor_lock,
            ));
    }
}

#[derive(Resource, Default)]
struct CameraState {
    cursor_locked: bool,
}

#[derive(Component)]
pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-10.0, 10.0, -10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController::default(),
    ));
}

fn camera_controller(
    time: Res<Time>,
    camera_state: Res<CameraState>,
    mut mouse_motion: EventReader<MouseMotion>,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    for (mut transform, mut controller) in query.iter_mut() {
        // Mouse look (only when cursor is locked)
        if camera_state.cursor_locked {
            for ev in mouse_motion.read() {
                controller.pitch -= ev.delta.y * controller.sensitivity;
                controller.yaw -= ev.delta.x * controller.sensitivity;
            }
            
            // Clamp pitch to prevent camera flipping
            controller.pitch = controller.pitch.clamp(-1.5, 1.5);
            
            // Apply rotation
            let rotation = Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
            transform.rotation = rotation;
        }

        // Keyboard movement
        let mut velocity = Vec3::ZERO;
        let forward = transform.forward();
        let right = transform.right();
        let up = Vec3::Y;

        // Get movement input
        if keyboard.pressed(KeyCode::W) {
            velocity += forward;
        }
        if keyboard.pressed(KeyCode::S) {
            velocity += -forward;
        }
        if keyboard.pressed(KeyCode::A) {
            velocity += -right;
        }
        if keyboard.pressed(KeyCode::D) {
            velocity += right;
        }
        if keyboard.pressed(KeyCode::Space) {
            velocity += up;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            velocity += -up;
        }

        // Apply movement
        if velocity != Vec3::ZERO {
            transform.translation += velocity.normalize() * controller.speed * time.delta_seconds();
        }
    }
}

fn toggle_cursor_lock(
    mut camera_state: ResMut<CameraState>,
    mut windows: Query<&mut Window>,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
) {
    let mut window = windows.single_mut();

    if keyboard.just_pressed(KeyCode::Escape) {
        camera_state.cursor_locked = false;
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }

    if mouse.just_pressed(MouseButton::Left) && !camera_state.cursor_locked {
        camera_state.cursor_locked = true;
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
}