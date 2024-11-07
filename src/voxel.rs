// src/voxel.rs
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use std::collections::HashMap;
use crate::render::BillboardPlugin;
use crate::voxel_types::{Voxel, VoxelRenderSettings};

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelRenderSettings>()
            .init_resource::<LodSettings>()
            .add_plugins(BillboardPlugin)
            .add_systems(Startup, setup_voxel_scene)
            .add_systems(Update, (
                update_chunk_visibility,
                update_voxel_lod,
                apply_occlusion_culling,
            ));
    }
}

pub const CHUNK_SIZE: i32 = 16;

// Local position within a chunk
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct LocalPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl LocalPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_vec3(vec: Vec3) -> Self {
        Self {
            x: vec.x as i32,
            y: vec.y as i32,
            z: vec.z as i32,
        }
    }
}

#[derive(Component, Debug)]
pub struct VoxelChunk {
    pub position: IVec3,
    pub voxels: Vec<Voxel>,
    pub bounds: Aabb,
    pub visible: bool,
    pub lod_level: usize,
}

impl VoxelChunk {
    pub fn new(position: IVec3, voxels: Vec<Voxel>) -> Self {
        // Calculate chunk bounds
        let min = Vec3::new(
            position.x as f32 * CHUNK_SIZE as f32,
            position.y as f32 * CHUNK_SIZE as f32,
            position.z as f32 * CHUNK_SIZE as f32,
        );
        let max = min + Vec3::splat(CHUNK_SIZE as f32);
        let bounds = Aabb::from_min_max(min, max);

        Self {
            position,
            voxels,
            bounds,
            visible: true,
            lod_level: 0,
        }
    }

    pub fn get_voxel_world_position(&self, voxel: &Voxel, voxel_size: f32) -> Vec3 {
        Vec3::new(
            (self.position.x * CHUNK_SIZE) as f32 + voxel.position.x,
            (self.position.y * CHUNK_SIZE) as f32 + voxel.position.y,
            (self.position.z * CHUNK_SIZE) as f32 + voxel.position.z,
        ) * voxel_size
    }

    // Add occlusion culling method
    pub fn filter_occluded_voxels(&mut self) {
        use std::collections::HashSet;
        
        // Create a hashset of positions for quick lookups
        let position_set: HashSet<LocalPos> = self.voxels
            .iter()
            .map(|v| LocalPos::from_vec3(v.position))
            .collect();

        // Keep only voxels that have at least one exposed face
        self.voxels.retain(|voxel| {
            let pos = LocalPos::from_vec3(voxel.position);
            
            // Check all six adjacent positions
            let adjacent_positions = [
                LocalPos::new(pos.x + 1, pos.y, pos.z), // Right
                LocalPos::new(pos.x - 1, pos.y, pos.z), // Left
                LocalPos::new(pos.x, pos.y + 1, pos.z), // Up
                LocalPos::new(pos.x, pos.y - 1, pos.z), // Down
                LocalPos::new(pos.x, pos.y, pos.z + 1), // Front
                LocalPos::new(pos.x, pos.y, pos.z - 1), // Back
            ];

            // A voxel is visible if any adjacent position is empty (no voxel present)
            // or is outside the chunk bounds
            adjacent_positions.iter().any(|adj_pos| {
                // Check if position is outside chunk bounds
                if adj_pos.x < 0 || adj_pos.x >= CHUNK_SIZE ||
                   adj_pos.y < 0 || adj_pos.y >= CHUNK_SIZE ||
                   adj_pos.z < 0 || adj_pos.z >= CHUNK_SIZE 
                {
                    return true; // Voxel is exposed at chunk boundary
                }

                // Check if there's no voxel at this position
                !position_set.contains(adj_pos)
            })
        });
    }
}

#[derive(Resource)]
pub struct LodSettings {
    pub distances: Vec<(f32, f32)>,
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            distances: vec![
                (0.0, 1.0),
                (50.0, 2.0),
                (100.0, 4.0),
            ],
        }
    }
}

fn setup_voxel_scene(mut commands: Commands) {
    // Setup lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Create a single chunk for testing
    let mut voxels = Vec::new();
    
    // Create a 15x15x15 cube of voxels
    for x in 0..15 {
        for y in 0..15 {
            for z in 0..15 {
                let pos = Vec3::new(x as f32, y as f32, z as f32);
                
                // Create gradient color based on position
                let color = Color::hsl(
                    (pos.x.atan2(pos.z).to_degrees() + 180.0) / 360.0 * 360.0,
                    (pos.y / 15.0 * 0.5 + 0.5).clamp(0.2, 1.0),
                    (1.0 - (pos - Vec3::splat(7.5)).length() / 15.0 * 0.5).clamp(0.3, 0.7),
                );

                voxels.push(Voxel {
                    position: pos,
                    color,
                });
            }
        }
    }

    info!("Created cube with {} voxels", voxels.len());
    
    // Create chunk and apply occlusion culling before spawning
    let mut chunk = VoxelChunk::new(IVec3::ZERO, voxels);
    chunk.filter_occluded_voxels();
    info!("After occlusion culling: {} voxels", chunk.voxels.len());
    
    commands.spawn(chunk);
}

// System to apply occlusion culling when chunks are modified
pub fn apply_occlusion_culling(
    mut chunks: Query<&mut VoxelChunk, Changed<VoxelChunk>>,
) {
    for mut chunk in chunks.iter_mut() {
        chunk.filter_occluded_voxels();
    }
}

fn update_chunk_visibility(
    mut chunks: Query<(&mut VoxelChunk, &GlobalTransform)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    settings: Res<VoxelRenderSettings>,
) {
    if let Ok((camera, camera_transform)) = camera.get_single() {
        let view_projection = camera.projection_matrix() * camera_transform.compute_matrix();
        
        for (mut chunk, transform) in chunks.iter_mut() {
            let chunk_center = transform.translation();
            let radius = (CHUNK_SIZE as f32) * 0.866; // Approximate radius of chunk
            
            // Distance-based culling
            let distance = (chunk_center - camera_transform.translation()).length();
            if distance > settings.render_distance {
                chunk.visible = false;
                continue;
            }
            
            // Frustum culling
            let view_space_pos = view_projection * chunk_center.extend(1.0);
            chunk.visible = view_space_pos.w > 0.0 && 
                           view_space_pos.x.abs() <= view_space_pos.w + radius &&
                           view_space_pos.y.abs() <= view_space_pos.w + radius &&
                           view_space_pos.z >= -radius;
        }
    }
}

fn update_voxel_lod(
    mut chunks: Query<(&mut VoxelChunk, &GlobalTransform)>,
    camera: Query<&Transform, With<Camera>>,
    settings: Res<LodSettings>,
) {
    if let Ok(camera_transform) = camera.get_single() {
        let camera_pos = camera_transform.translation;
        
        for (mut chunk, transform) in chunks.iter_mut() {
            let distance = (transform.translation() - camera_pos).length();
            
            // Update LOD level based on distance
            for (i, (threshold, _)) in settings.distances.iter().enumerate() {
                if distance <= *threshold {
                    chunk.lod_level = i;
                    break;
                }
            }
        }
    }
}