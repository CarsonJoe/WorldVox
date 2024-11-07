// src/render/billboard.rs
use bevy::{
    prelude::*,
    render::{render_resource::*, mesh::*},
};

use crate::voxel::{VoxelChunk};
use crate::voxel_types::VoxelRenderSettings;

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BillboardAssets>()
            .add_systems(Startup, setup_billboard_assets)
            .add_systems(Update, update_billboards);
    }
}

#[derive(Component)]
struct BillboardMarker;

#[derive(Resource, Default)]
struct BillboardAssets {
    circle_texture: Option<Handle<Image>>,
}

fn create_circle_texture(images: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    let size = 64u32;
    let mut texture_data = Vec::with_capacity((size * size * 4) as usize);
    
    for y in 0..size {
        for x in 0..size {
            let distance = Vec2::new(
                (x as f32 / size as f32 - 0.5) * 2.0,
                (y as f32 / size as f32 - 0.5) * 2.0
            ).length();
            
            let alpha = if distance <= 0.95 {
                1.0
            } else if distance <= 1.0 {
                1.0 - (distance - 0.95) / 0.05
            } else {
                0.0
            };
            
            texture_data.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }

    let texture = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );

    images.add(texture)
}

fn create_billboard_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    let vertices = vec![
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];

    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];

    let uvs = vec![
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
    ];

    let indices = vec![0, 2, 1, 0, 3, 2];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}

fn setup_billboard_assets(
    mut images: ResMut<Assets<Image>>,
    mut billboard_assets: ResMut<BillboardAssets>,
) {
    let texture_handle = create_circle_texture(&mut images);
    billboard_assets.circle_texture = Some(texture_handle);
}

fn update_billboards(
    mut commands: Commands,
    settings: Res<VoxelRenderSettings>,
    chunks: Query<&VoxelChunk>,
    camera: Query<&Transform, With<Camera>>,
    old_billboards: Query<Entity, With<BillboardMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    billboard_assets: Res<BillboardAssets>,
) {
    // Remove old billboards
    for entity in old_billboards.iter() {
        commands.entity(entity).despawn();
    }

    // Don't render if in debug mode
    if settings.debug_mode {
        return;
    }

    let camera_transform = camera.single();
    let mesh_handle = meshes.add(create_billboard_mesh());

    if let Some(circle_texture) = &billboard_assets.circle_texture {
        for chunk in chunks.iter() {
            if !chunk.visible {
                continue;
            }

            for voxel in &chunk.voxels {
                let world_pos = Vec3::new(
                    (chunk.position.x as f32 * settings.voxel_size) + (voxel.position.x * settings.voxel_size),
                    (chunk.position.y as f32 * settings.voxel_size) + (voxel.position.y * settings.voxel_size),
                    (chunk.position.z as f32 * settings.voxel_size) + (voxel.position.z * settings.voxel_size),
                );

                let to_camera = (camera_transform.translation - world_pos).normalize();
                let camera_up = camera_transform.local_y();
                let right = camera_up.cross(-to_camera).normalize();
                let up = (-to_camera).cross(right).normalize();
                let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, -to_camera));

                let material = materials.add(StandardMaterial {
                    base_color: voxel.color,
                    base_color_texture: Some(circle_texture.clone()),
                    alpha_mode: AlphaMode::Mask(0.1),
                    unlit: true,
                    double_sided: true,
                    ..default()
                });

                commands.spawn((
                    PbrBundle {
                        mesh: mesh_handle.clone(),
                        material,
                        transform: Transform {
                            translation: world_pos,
                            rotation,
                            scale: Vec3::splat(settings.voxel_size * 2.0),
                        },
                        ..default()
                    },
                    BillboardMarker,
                ));
            }
        }
    }
}