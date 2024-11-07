// src/main.rs
use bevy::prelude::*;

mod voxel;
mod voxel_types;
mod render;
mod camera;
mod diagnostics;

use voxel::VoxelPlugin;
use camera::CameraPlugin;
use diagnostics::DiagnosticsPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voxel Engine".into(),
                    ..default()
                }),
                ..default()
            }),
            VoxelPlugin,
            CameraPlugin,
            DiagnosticsPlugin,
        ))
        .run();
}