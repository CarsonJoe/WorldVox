use bevy::prelude::*;
use std::collections::HashMap;
use crate::voxel::{VoxelChunk, CHUNK_SIZE};
use crate::voxel_types::Voxel;

// Represents a position in the chunk's local coordinate system
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct LocalPos {
    x: i32,
    y: i32,
    z: i32,
}

impl LocalPos {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn from_vec3(vec: Vec3) -> Self {
        Self {
            x: vec.x as i32,
            y: vec.y as i32,
            z: vec.z as i32,
        }
    }
}

impl VoxelChunk {
    // Add this method to VoxelChunk to filter out occluded voxels
    pub fn filter_occluded_voxels(&mut self) {
        // Create a hashmap for quick position lookups
        let position_map: HashMap<LocalPos, &Voxel> = self.voxels
            .iter()
            .map(|v| (LocalPos::from_vec3(v.position), v))
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
                !position_map.contains_key(adj_pos)
            })
        });
    }
}

// System to apply occlusion culling during chunk creation or modification
pub fn apply_occlusion_culling(
    mut chunks: Query<&mut VoxelChunk, Changed<VoxelChunk>>,
) {
    for mut chunk in chunks.iter_mut() {
        chunk.filter_occluded_voxels();
    }
}