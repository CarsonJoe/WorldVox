// src/voxel_types.rs
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Voxel {
    pub position: Vec3,
    pub color: Color,
}

#[derive(Resource)]
pub struct VoxelRenderSettings {
    pub debug_mode: bool,
    pub voxel_size: f32,
    pub render_distance: f32,
    pub show_chunk_bounds: bool,
    pub show_diagnostics: bool,
}

impl Default for VoxelRenderSettings {
    fn default() -> Self {
        Self {
            debug_mode: false,
            voxel_size: 1.0,
            render_distance: 100.0,
            show_chunk_bounds: false,
            show_diagnostics: true,
        }
    }
}