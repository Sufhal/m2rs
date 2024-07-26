use std::{any::Any, io::{BufReader, Cursor}};
use gltf::json::extensions::mesh;
use wgpu::util::DeviceExt;

use crate::modules::core::model::{Material, Mesh, Model, ModelVertex};
use super::assets::{load_binary, load_string};

pub async fn load_model_glb(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<Model> {
    let gltf_bin = load_binary(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_bin);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = gltf::Gltf::from_reader(gltf_reader)?;

    let mut buffer_data = extract_buffer_data(&gltf).await?;
    let mut materials = Vec::<Material>::new();
    let mut meshes = extract_meshes(device, file_name, gltf.scenes(), &buffer_data);

    Ok(Model {
        meshes,
        materials,
    })
}

pub async fn load_model_gltf(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<Model> {
    let gltf_text = load_string(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_text);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = gltf::Gltf::from_reader(gltf_reader)?;

    let mut buffer_data = extract_buffer_data(&gltf).await?;
    let mut materials = Vec::<Material>::new();
    let mut meshes = extract_meshes(device, file_name, gltf.scenes(), &buffer_data);

    Ok(Model {
        meshes,
        materials,
    })
}

async fn extract_buffer_data(
    model: &gltf::Gltf
) -> anyhow::Result<Vec<Vec<u8>>> {
    let blob = &model.blob;
    let mut buffer_data = Vec::new();
    for buffer in model.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                if let Some(blob) = blob.as_deref() {
                    buffer_data.push(blob.into());
                };
            }
            gltf::buffer::Source::Uri(uri) => {
                let bin = load_binary(uri).await?;
                buffer_data.push(bin);
            }
        }
    }
    Ok(buffer_data)
}

fn extract_meshes(
    device: &wgpu::Device, 
    file_name: &str, 
    scenes: gltf::iter::Scenes,
    buffer_data: &Vec<Vec<u8>>
) -> Vec<Mesh> {

    let mut meshes = Vec::<Mesh>::new();

    fn extract_from_node(node: &gltf::Node, device: &wgpu::Device, meshes: &mut Vec<Mesh>, buffer_data: &Vec<Vec<u8>>, file_name: &str) {
        if let Some(mesh) = node.mesh() {
            let primitives = mesh.primitives();
            primitives.for_each(|primitive| {

                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let mut vertices = Vec::new();
                if let Some(vertex_attribute) = reader.read_positions() {
                    vertex_attribute.for_each(|vertex| {
                        // dbg!(vertex);
                        vertices.push(ModelVertex {
                            position: vertex,
                            tex_coords: Default::default(),
                            normal: Default::default(),
                        })
                    });
                }
                if let Some(normal_attribute) = reader.read_normals() {
                    let mut normal_index = 0;
                    normal_attribute.for_each(|normal| {
                        // dbg!(normal);
                        vertices[normal_index].normal = normal;

                        normal_index += 1;
                    });
                }
                if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                    let mut tex_coord_index = 0;
                    tex_coord_attribute.for_each(|tex_coord| {
                        // dbg!(tex_coord);
                        vertices[tex_coord_index].tex_coords = tex_coord;

                        tex_coord_index += 1;
                    });
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    // dbg!(indices_raw);
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }
                // dbg!(indices);

                // println!("{:#?}", &indices.expect("got indices").data_type());
                // println!("{:#?}", &indices.expect("got indices").index());
                // println!("{:#?}", &material);

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                meshes.push(Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    // material: m.mesh.material_id.unwrap_or(0),
                    material: 0,
                });
            });
        }
    }

    fn traverse(node: &gltf::Node, device: &wgpu::Device, meshes: &mut Vec<Mesh>, buffer_data: &Vec<Vec<u8>>, file_name: &str) {
        println!("traversing node {}", node.index());
        extract_from_node(node, device, meshes, buffer_data, file_name);
        for children in node.children() {
            traverse(&children, device, meshes, buffer_data, file_name);
        }
    }
    
    for scene in scenes {
        for node in scene.nodes() {
            traverse(&node, device, &mut meshes, buffer_data, file_name);
        }
    }

    meshes
}