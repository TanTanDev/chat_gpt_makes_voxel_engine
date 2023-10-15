use bevy::{
    math::{ivec3, vec3, IVec3},
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bracket_noise::prelude::*;
use std::{collections::HashMap, f32::consts::PI};
pub mod camera;
mod material;
use camera::*;
use material::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(CustomMaterialPlugin)
        .insert_resource(Msaa::default())
        .insert_resource(ChunkMap(HashMap::new()))
        .add_startup_system(setup)
        .add_system(loader_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI / 2.,
            -PI / 4.,
        )),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 5.0 }.into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 7.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCamera::new(5.0, 10.0),
        ChunkLoader {
            player_position: ChunkLocation(ivec3(0, 0, 0)),
            loaded_chunks: vec![],
            chunk_entities: HashMap::new(),
        },
    ));
}

#[derive(Debug)]
struct Voxel {
    is_solid: bool,
}

#[derive(Debug)]
struct Chunk {
    voxels: Vec<Voxel>,
}

#[derive(Debug, Resource)]
pub struct ChunkMap(HashMap<IVec3, Chunk>);

fn generate_voxel_data(chunk_pos: IVec3) -> Vec<Voxel> {
    let mut voxels = Vec::with_capacity(32 * 32 * 32);
    let mut noise = FastNoise::seeded(1234);
    noise.set_noise_type(NoiseType::PerlinFractal);
    noise.set_frequency(0.04);

    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let pos = IVec3::new(
                    x + chunk_pos.x * 32,
                    y + chunk_pos.y * 32,
                    z + chunk_pos.z * 32,
                );
                // let is_solid = noise.get_noise3d(pos.x as f32, pos.y as f32, pos.z as f32) > 0.0;
                let height = noise.get_noise3d(pos.x as f32, pos.y as f32, pos.z as f32);
                // + noise.get_noise(pos.x as f32 / 20.0, pos.z as f32 / 20.0);
                let is_solid = (pos.y as f32) < ((height + 1.0) * 0.5) * 20.0;
                voxels.push(Voxel { is_solid });
            }
        }
    }

    voxels
}

fn generate_mesh(
    chunk_pos: IVec3,
    chunk: &Chunk,
    // chunk_map: &HashMap<IVec3, Chunk>,
) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 4]>, Vec<[f32; 2]>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut colors = Vec::new();
    let mut uvs = Vec::new();

    let chunk_offset = Vec3::new(
        chunk_pos.x as f32 * 32.0,
        chunk_pos.y as f32 * 32.0,
        chunk_pos.z as f32 * 32.0,
    );

    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let index = z * 1024 + y * 32 + x;

                if !chunk.voxels[index].is_solid {
                    continue;
                }

                let pos = Vec3::new(
                    x as f32 + chunk_offset.x,
                    y as f32 + chunk_offset.y,
                    z as f32 + chunk_offset.z,
                );

                // Generate vertices and indices for the cube at this position
                let cube_indices = generate_cube_indices(vertices.len() as u32);
                indices.extend(cube_indices);
                let cube_vertices = generate_cube_vertices(pos);
                vertices.extend(cube_vertices);

                colors.extend([[0.0, 1.0, 0.0, 1.0]; 8]);
                uvs.extend([[1.0, 0.0]; 8]);
            }
        }
    }

    (vertices, indices, colors, uvs)
}

pub const CHUNK_SIZE: usize = 32;

// fn generate_
fn generate_cube_vertices(pos: Vec3) -> Vec<[f32; 3]> {
    let x = pos.x;
    let y = pos.y;
    let z = pos.z;
    vec![
        [x + 0.0, y + 1.0, z + 1.0], // 0
        [x + 1.0, y + 1.0, z + 1.0], // 1
        [x + 1.0, y + 1.0, z + 0.0], // 2
        [x + 0.0, y + 1.0, z + 0.0], // 3
        [x + 0.0, y + 0.0, z + 0.0], // 4
        [x + 1.0, y + 0.0, z + 0.0], // 5
        [x + 1.0, y + 0.0, z + 1.0], // 6
        [x + 0.0, y + 0.0, z + 1.0], // 7
    ]
}

// vertex index, instance_index
// normal is sampled from: instance_index / 6
fn generate_cube_normals() -> [Vec3; 6] {
    [
        Vec3::Y,  // Top,
        -Vec3::Y, // Bottom,
        -Vec3::X, // Left,
        Vec3::X,  // Right,
        Vec3::Z,  // Front,
        -Vec3::Z, // Back,
    ]
}

fn generate_cube_indices(start_index: u32) -> Vec<u32> {
    Face::all_variants()
        .iter()
        .map(|face| face.indices())
        .flatten()
        .map(|index| index + start_index)
        .collect()
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
struct ChunkLocation(pub IVec3);

#[derive(Component)]
pub struct ChunkLoader {
    player_position: ChunkLocation,
    loaded_chunks: Vec<ChunkLocation>,
    chunk_entities: HashMap<ChunkLocation, Entity>,
}

pub fn loader_system(
    mut query: Query<&mut ChunkLoader>,
    mut cameras: Query<&mut Transform, With<Camera>>,
    mut mesh_map: ResMut<ChunkMap>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(camera_transform) = cameras.get_single_mut() else {
        return;
    };
    for mut chunk_loader in query.iter_mut() {
        let camera_pos = camera_transform.translation;
        let camera_chunk = ivec3(
            camera_pos.x as i32 >> 5,
            camera_pos.y as i32 >> 5,
            camera_pos.z as i32 >> 5,
        );

        let view_dist = 1;
        chunk_loader.update_player_position(
            ChunkLocation(camera_chunk),
            view_dist,
            &mut mesh_map.0,
            &mut commands,
            &mut materials,
            &mut custom_materials,
            &mut meshes,
        );
    }
}

impl ChunkLoader {
    fn new(player_position: ChunkLocation) -> ChunkLoader {
        ChunkLoader {
            player_position,
            loaded_chunks: vec![],
            chunk_entities: HashMap::new(),
        }
    }

    fn update_player_position(
        &mut self,
        new_position: ChunkLocation,
        view_distance: i32,
        chunks: &mut HashMap<IVec3, Chunk>,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        custom_materials: &mut ResMut<Assets<CustomMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        let old_chunk_coords = self.player_position;
        let new_chunk_coords = new_position;

        // Check if player has moved to a new chunk
        if old_chunk_coords != new_chunk_coords {
            println!("loading in a chunk");
            // Load chunks in range of player position
            let chunks_to_load = self.get_chunks_to_load(new_position, view_distance);
            for chunk_coords in chunks_to_load {
                self.load_chunk(
                    chunk_coords,
                    chunks,
                    commands,
                    materials,
                    custom_materials,
                    meshes,
                );
            }

            // Unload chunks that are no longer in range
            let chunks_to_unload = self.get_chunks_to_unload(old_chunk_coords, view_distance);
            for chunk_coords in chunks_to_unload {
                self.unload_chunk(chunk_coords, chunks, commands);
            }

            // Update player position
            self.player_position = new_position;
        }
    }

    fn get_chunks_to_load(
        &self,
        position: ChunkLocation,
        view_distance: i32,
    ) -> Vec<ChunkLocation> {
        let mut chunks_to_load = vec![];
        for x in position.0.x - view_distance..=position.0.x + view_distance {
            for y in position.0.y - view_distance..=position.0.y + view_distance {
                for z in position.0.z - view_distance..=position.0.z + view_distance {
                    let chunk_coords = ChunkLocation(ivec3(x, y, z));
                    if !self.loaded_chunks.contains(&chunk_coords) {
                        chunks_to_load.push(chunk_coords);
                    }
                }
            }
        }
        chunks_to_load
    }

    fn get_chunks_to_unload(
        &self,
        position: ChunkLocation,
        view_distance: i32,
    ) -> Vec<ChunkLocation> {
        let mut chunks_to_unload = vec![];
        for chunk_coords in &self.loaded_chunks {
            let distance = (chunk_coords.0.x - position.0.x).abs()
                + (chunk_coords.0.y - position.0.y).abs()
                + (chunk_coords.0.z - position.0.z).abs();
            if distance > view_distance {
                chunks_to_unload.push(*chunk_coords);
            }
        }
        chunks_to_unload
    }

    fn load_chunk(
        &mut self,
        chunk_coords: ChunkLocation,
        chunks: &mut HashMap<IVec3, Chunk>,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        custom_materials: &mut ResMut<Assets<CustomMaterial>>,
        mut meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        let voxels = generate_voxel_data(chunk_coords.0);
        let chunk = Chunk { voxels };
        let (vertices, indices, colors, uvs) = generate_mesh(chunk_coords.0, &chunk);
        chunks.insert(chunk_coords.0, chunk);
        self.loaded_chunks.push(chunk_coords);

        let chunk_pos = IVec3::new(0, 0, 0);
        let voxels = generate_voxel_data(chunk_pos);
        let chunk = Chunk { voxels };

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(indices)));

        let id = commands
            .spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(mesh),
                    material: custom_materials.add(CustomMaterial {
                        normals: generate_cube_normals(),
                    }),
                    ..Default::default()
                },
                Wireframe,
            ))
            .id();

        self.chunk_entities.insert(chunk_coords, id);

        println!(
            "Loaded chunk at ({}, {}, {})",
            chunk_coords.0.x, chunk_coords.0.y, chunk_coords.0.z
        );
    }

    fn unload_chunk(
        &mut self,
        chunk_coords: ChunkLocation,
        chunks: &mut HashMap<IVec3, Chunk>,
        commands: &mut Commands,
    ) {
        let index = self.loaded_chunks.iter().position(|&c| c == chunk_coords);
        if let Some(entity) = self.chunk_entities.remove(&chunk_coords) {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(i) = index {
            self.loaded_chunks.remove(i);
            println!(
                "Unloaded chunk at ({}, {}, {})",
                chunk_coords.0.x, chunk_coords.0.y, chunk_coords.0.z
            );
        }
    }
}

#[derive(Clone, Copy)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Face {
    // pub fn all_variants() -> [Face; 6] {
    pub fn all_variants() -> Vec<Face> {
        vec![
            Self::Top,
            Self::Bottom,
            Self::Left,
            Self::Right,
            Self::Front,
            Self::Back,
        ]
    }
    pub fn indices(&self) -> [u32; 6] {
        match self {
            Face::Top => [0, 1, 2, 2, 3, 0],
            Face::Bottom => [5, 7, 6, 5, 4, 7],
            Face::Left => [7, 0, 4, 4, 0, 3],
            Face::Right => [6, 5, 1, 1, 5, 2],
            Face::Front => [7, 1, 0, 7, 6, 1],
            Face::Back => [5, 4, 3, 3, 2, 5],
        }
    }
    pub fn normal(&self) -> [f32; 3] {
        match self {
            Face::Top => [0.0, 1.0, 0.0],
            Face::Bottom => [0.0, -1.0, 0.0],
            Face::Left => [-1.0, 0.0, 0.0],
            Face::Right => [1.0, 0.0, 0.0],
            Face::Front => [0.0, 0.0, 1.0],
            Face::Back => [0.0, 0.0, -1.0],
        }
    }
}
