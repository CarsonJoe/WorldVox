// src/diagnostics.rs
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use crate::voxel::VoxelChunk;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<PerformanceStats>()
            .add_systems(Startup, setup_diagnostics)
            .add_systems(Update, (
                update_performance_stats,
                update_diagnostics_text,
            ).chain());
    }
}

#[derive(Resource, Default)]
struct PerformanceStats {
    voxels_rendered: usize,
    visible_chunks: usize,
    camera_position: Vec3,
    frame_time: f64,
    fps: f64,
}

#[derive(Component)]
struct DiagnosticsText;

fn setup_diagnostics(mut commands: Commands) {
    // Spawn diagnostics text overlay
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Diagnostics\n",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 18.0,
                color: Color::YELLOW,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        DiagnosticsText,
    ));
}

fn update_performance_stats(
    mut stats: ResMut<PerformanceStats>,
    diagnostics: Res<DiagnosticsStore>,
    chunks: Query<&VoxelChunk>,
    camera: Query<&Transform, With<Camera>>,
) {
    // Update voxel count
    stats.voxels_rendered = chunks
        .iter()
        .filter(|chunk| chunk.visible)
        .map(|chunk| chunk.voxels.len())
        .sum();
    
    stats.visible_chunks = chunks
        .iter()
        .filter(|chunk| chunk.visible)
        .count();
    
    // Update camera position
    if let Ok(camera_transform) = camera.get_single() {
        stats.camera_position = camera_transform.translation;
    }
    
    // Update FPS and frame time
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps.smoothed() {
            stats.fps = fps_value;
        }
    }
    
    if let Some(frame_time) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(frame_time_value) = frame_time.smoothed() {
            stats.frame_time = frame_time_value;
        }
    }
}

fn update_diagnostics_text(
    stats: Res<PerformanceStats>,
    mut query: Query<&mut Text, With<DiagnosticsText>>,
) {
    for mut text in &mut query {
        text.sections[1].value = format!(
            "FPS: {:.1}\nFrame Time: {:.2}ms\nVoxels Rendered: {}\nVisible Chunks: {}\nCamera Pos: {:.1} {:.1} {:.1}\n",
            stats.fps,
            stats.frame_time,
            stats.voxels_rendered,
            stats.visible_chunks,
            stats.camera_position.x,
            stats.camera_position.y,
            stats.camera_position.z,
        );
    }
}