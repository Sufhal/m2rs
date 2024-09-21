use cgmath::{Matrix4, Rotation3};
use wgpu::util::DeviceExt;
use crate::modules::{assets::assets::load_binary, core::{model::{CustomMesh, Transformable}, object_3d::Object3D, texture::Texture}, geometry::plane::Plane, state::State, utils::functions::u8_to_string_with_len};
use super::{areadata::AreaData, height::Height, setting::Setting, texture_set::ChunkTextureSet, water::{Water, WaterTexture}};

const SIZE: f32 = 256.0;

pub struct Chunk {
    pub name: String,
    pub terrain_mesh: CustomMesh,
    pub terrain_plane: Plane,
    pub water_mesh: CustomMesh,
    pub depth_buffer: wgpu::Buffer,
    pub mean_height: f32,
    water_buffer: wgpu::Buffer,
    area_data: AreaData,
    limits: ([f32; 2], [f32; 2]), // ([minX, maxX], [minY, maxY]) 
}

impl Chunk {
    pub async fn new(
        terrain_path: &str, 
        x: &u8, 
        y: &u8,
        setting: &Setting,
        textures: &Vec<Texture>,
        water_textures: &WaterTexture,
        state: &mut State<'_>
    ) -> anyhow::Result<Self> {
        let name = Self::name_from(*x, *y);
        let chunk_path = &format!("{terrain_path}/{name}");
        let height = Height::read(&chunk_path, setting.height_scale).await?;
        let mean_height = height.vertices.iter().fold(0.0, |acc, v| acc + *v) / height.vertices.len() as f32;
        let water = Water::read(&chunk_path).await?;
        let textures_set = ChunkTextureSet::read(&chunk_path).await?;
        let alpha_atlas = {
            let raw = load_binary(&format!("{chunk_path}/tiles_atlas.raw")).await?;
            Texture::from_raw_bytes(
                &raw, 
                768, 
                768, 
                wgpu::TextureFormat::R8Unorm, 
                wgpu::FilterMode::Nearest,
                768, 
                state
            )
        };
        let segments = 128u32;
        let size = segments as f32 * 2.0;
        let position = [
            (*x as f32 * size) + (size / 2.0),
            0.0,
            (*y as f32 * size) + (size / 2.0)
        ];
        let mut terrain_plane = Plane::new(size, size, segments, segments);
        terrain_plane.set_vertices_height(&height.vertices);
        terrain_plane.vertices.apply_matrix(&Matrix4::from_translation(position.into()));
        terrain_plane.update_normals();
        let terrain_mesh = terrain_plane.to_terrain_mesh(
            &state.device, 
            &state.terrain_pipeline, 
            name.clone(),
            textures,
            &alpha_atlas,
            &textures_set
        );
        let water_plane = water.generate_plane(setting.height_scale);
        let depth = Water::calculate_depth(&water_plane, &terrain_plane);
        let depth_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Depth Buffer"),
            contents: bytemuck::cast_slice(&depth),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let (water_mesh, water_buffer) = water_plane.to_water_mesh(
            &state.device, 
            &state.water_pipeline, 
            name.clone(),
            [
                (*x as f32 * size),
                0.0,
                (*y as f32 * size)
            ],
            &water_textures,
        );
        let area_data = AreaData::read(&chunk_path).await?;
        let limits = (
            [*x as f32 * SIZE, (*x as f32 + 1.0) * SIZE],
            [*y as f32 * SIZE, (*y as f32 + 1.0) * SIZE],
        );
        Ok(Self {
            name,
            terrain_mesh,
            terrain_plane,
            water_mesh,
            area_data,
            water_buffer,
            depth_buffer,
            mean_height,
            limits,
        })
    }

    pub fn get_properties_to_preload(&self) -> Vec<String> {
        self.area_data.objects
            .iter()
            .map(|v| v.id.clone())
            .collect::<Vec<_>>()
    }

    pub async fn load_objects_instances(&mut self, state: &mut State<'_>) {
        // every objects of the chunk should be loaded at this time
        // request an instance of them using its property id as object's name
        for area_object in &self.area_data.objects {
            for scene_object in state.scene.get_all_objects_mut() {
                if let Some(object3d) = &mut scene_object.object3d {
                    if scene_object.name.as_ref() == Some(&area_object.id) {
                        match object3d {
                            Object3D::Simple(simple) => {
                                let instance = simple.request_instance(&state.device);
                                instance.set_position(cgmath::Vector3::from([
                                    area_object.position[0],
                                    area_object.position[1] + area_object.offset,
                                    area_object.position[2]
                                ]));
                                instance.set_rotation(cgmath::Quaternion::from_angle_y(cgmath::Rad::from(cgmath::Deg(area_object.rotation[1]))));
                                instance.take();
                            },
                            _ => ()
                        };
                    } 
                }
            }
        }
    }

    /// Something like "001002", "004005"
    pub fn name_from(x: u8, y: u8) -> String {
        u8_to_string_with_len(x, 3) + &u8_to_string_with_len(y, 3)
    }

    pub fn update(&self, water_texture: &WaterTexture, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.water_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&[water_texture.uniform]),
        );
    }

	pub fn contains_position(&self, position: &[f32; 3]) -> bool {
        let [x, _, y] = position;
        let ([x_min, x_max], [y_min, y_max]) = self.limits;
		(*x >= x_min && *x < x_max) && (*y >= y_min && *y < y_max)
	}


}


#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct ChunkInformationUniform {
    pub textures_count: u32
}