use cgmath::{InnerSpace, Matrix4, Vector3};
use rustc_hash::FxHashMap;
use wgpu::util::DeviceExt;
use crate::modules::{core::{model::{CustomMesh, SimpleVertex, TransformUniform}, texture::Texture}, environment::clouds::CloudsUniform, pipelines::{clouds_pipeline::CloudsPipeline, sun_pipeline::SunPipeline, terrain_pipeline::TerrainPipeline, water_pipeline::WaterPipeline}, terrain::{chunk::ChunkInformationUniform, texture_set::ChunkTextureSet, water::WaterTexture}, utils::functions::add_normals};

#[derive(Debug)]
pub struct Plane {
    pub vertices: Vec<SimpleVertex>,
    pub indices: Vec<u32>,
    pub indices_positions: FxHashMap<usize, usize>,
}

impl Plane {
    pub fn new(width: f32, height: f32, segments_x: u32, segments_y: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut indices_positions = FxHashMap::default();
        let segment_width = width / segments_x as f32;
        let segment_height = height / segments_y as f32;
        let normal = [0.0, 1.0, 0.0]; // Normale pointant vers le haut

        for y in 0..=segments_y {
            for x in 0..=segments_x {
                let position = [
                    x as f32 * segment_width - width / 2.0,
                    0.0,
                    y as f32 * segment_height - height / 2.0,
                ];

                let tex_coords = [
                    x as f32 / segments_x as f32,
                    y as f32 / segments_y as f32,
                ];

                vertices.push(SimpleVertex::new(
                    position,
                    tex_coords,
                    normal,
                ));
            }
        }
        for y in 0..segments_y {
            for x in 0..segments_x {
                let i0 = y * (segments_x + 1) + x;
                let i1 = i0 + 1;
                let i2 = i0 + (segments_x + 1);
                let i3 = i2 + 1;

                indices_positions.insert(i0 as usize, indices.len());
                indices_positions.insert(i1 as usize, indices.len() + 3);
                indices_positions.insert(i2 as usize, indices.len() + 4);
                indices_positions.insert(i3 as usize, indices.len() + 5);

                indices.push(i0);
                indices.push(i2);
                indices.push(i1);

                indices.push(i1);
                indices.push(i2);
                indices.push(i3);

            }
        }
        Plane {
            vertices,
            indices,
            indices_positions
        }
    }

    pub fn from(positions: Vec<f32>, uvs: Vec<f32>, indices: Vec<u32>, indices_positions: FxHashMap<usize, usize>) -> Self {
        let mut vertices = Vec::new();
        let normal = [0.0, 1.0, 0.0]; // Normale pointant vers le haut
        for i in 0..positions.len() / 3 {
            let p = i * 3;
            let u = i * 2;
            vertices.push(SimpleVertex::new(
                [
                    positions[p + 0],
                    positions[p + 1],
                    positions[p + 2],
                ], 
                [
                    uvs[u + 0],
                    uvs[u + 1],
                ], 
                normal
            ))
        }
        Plane {
            vertices,
            indices,
            indices_positions
        }
    }

    pub fn set_vertices_height(&mut self, vertices_height: &Vec<f32>) {
        if vertices_height.len() != self.vertices.len() {
            panic!("Impossible to set vertices height with incompatible surface vertices count");
        }
        for i in 0..vertices_height.len() {
            self.vertices[i].position[1] = vertices_height[i];
        }
        self.update_normals();
    }

    fn update_normals(&mut self) {
        for vertex in &mut self.vertices {
            vertex.normal = [0.0, 0.0, 0.0];
        }
        for i in (0..self.indices.len()).step_by(3) {
            let i0 = self.indices[i] as usize;
            let i1 = self.indices[i + 1] as usize;
            let i2 = self.indices[i + 2] as usize;

            let v0 = Vector3::from(self.vertices[i0].position);
            let v1 = Vector3::from(self.vertices[i1].position);
            let v2 = Vector3::from(self.vertices[i2].position);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;

            let normal = edge1.cross(edge2).normalize();

            self.vertices[i0].normal = add_normals(self.vertices[i0].normal, normal);
            self.vertices[i1].normal = add_normals(self.vertices[i1].normal, normal);
            self.vertices[i2].normal = add_normals(self.vertices[i2].normal, normal);
        }

        for vertex in &mut self.vertices {
            let normal = Vector3::from(vertex.normal).normalize();
            vertex.normal = [normal.x, normal.y, normal.z];
        }
    }

    pub fn to_terrain_mesh(
        &self, 
        device: &wgpu::Device, 
        terrain_pipeline: &TerrainPipeline, 
        name: String, 
        position: [f32; 3], 
        textures: &Vec<Texture>,
        alpha_maps: &Vec<Texture>,
        textures_set: &ChunkTextureSet,
    ) -> CustomMesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(Matrix4::from_translation(position.into()).into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let chunk_informations_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Informations Buffer"),
            contents: bytemuck::cast_slice(&[ChunkInformationUniform { textures_count: textures_set.textures.len() as u32 }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let get_texture_view = |index: u8| {
            let set_index = textures_set.textures.get(index as usize).unwrap_or(&0);
            &textures[*set_index as usize].view
        };

        let get_alpha_view = |index: usize| {
            if let Some(texture) = alpha_maps.get(index) {
                &texture.view
            } else {
                &alpha_maps[0].view
            }
        };

        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: chunk_informations_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&textures[0].sampler),
            },
        ];

        for i in 0..8 {
            let offset = 3;
            entries.push(wgpu::BindGroupEntry {
                binding: offset + (i * 2) as u32,
                resource: wgpu::BindingResource::TextureView(get_texture_view(i)),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: offset + (i * 2 + 1) as u32,
                resource: wgpu::BindingResource::TextureView(get_alpha_view(i as usize)),
            });
        }

        let sampler_alpha = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        entries.push(wgpu::BindGroupEntry {
            binding: entries.len() as u32,
            resource: wgpu::BindingResource::Sampler(&sampler_alpha)
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &terrain_pipeline.bind_group_layouts.mesh,
            entries: &entries,
            label: None,
        });

        CustomMesh {
            name,
            transform_buffer,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            bind_group
        }
    }

    pub fn to_water_mesh(
        &self, 
        device: &wgpu::Device, 
        water_pipeline: &WaterPipeline, 
        name: String, 
        position: [f32; 3],
        water_texture: &WaterTexture,
    ) -> (CustomMesh, wgpu::Buffer) {
        // dbg!(&self.vertices.iter().map(|v| v.tex_coords).collect::<Vec<_>>());
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(Matrix4::from_translation(position.into()).into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let water_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Buffer"),
            contents: bytemuck::cast_slice(&[water_texture.uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: water_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&water_texture.atlas_texture.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&water_texture.atlas_texture.view),
            },
        ];

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &water_pipeline.bind_group_layouts.mesh,
            entries: &entries,
            label: None,
        });

        (
            CustomMesh {
                name,
                transform_buffer,
                vertex_buffer,
                index_buffer,
                num_elements: self.indices.len() as u32,
                bind_group
            },
            water_buffer
        )
    }

    pub fn to_sun_mesh(
        &self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue,
        sun_pipeline: &SunPipeline, 
        position: [f32; 3],
    ) -> CustomMesh {
        let texture = Texture::from_bytes(device, queue, include_bytes!("../../../assets/pack/environment/sun-sprite.png"), "sun").unwrap();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sun Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sun Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let matrix = Matrix4::from_translation(position.into());
        // let matrix = Matrix4::from_translation(position.into()) * Matrix4::from_angle_z(Rad(PI / 2.0));
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(matrix.into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            }
        ];

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sun_pipeline.bind_group_layouts.mesh,
            entries: &entries,
            label: None,
        });

        CustomMesh {
            name: "sun".to_string(),
            transform_buffer,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            bind_group
        }
    }

    pub fn to_clouds_mesh(
        &self, 
        device: &wgpu::Device, 
        clouds_pipeline: &CloudsPipeline, 
        position: [f32; 3],
        texture: &Texture,
        uniform: CloudsUniform,
    ) -> (CustomMesh, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clouds Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clouds Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(Matrix4::from_translation(position.into()).into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let clouds_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clouds Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: clouds_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
        ];

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &clouds_pipeline.bind_group_layouts.mesh,
            entries: &entries,
            label: None,
        });

        (
            CustomMesh {
                name: "clouds".to_string(),
                transform_buffer,
                vertex_buffer,
                index_buffer,
                num_elements: self.indices.len() as u32,
                bind_group
            },
            clouds_buffer
        )
    }


}

