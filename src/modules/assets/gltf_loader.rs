use std::{any::Any, collections::{HashMap, HashSet}, hash::Hash, io::{BufReader, Cursor}, ops::Deref};
use wgpu::util::DeviceExt;

use crate::modules::{assets::assets::{load_material, load_material_from_bytes}, core::{model::{Material, Mesh, Model, ModelVertex, TransformUniform}, object::Object, object_3d::{self, Object3D}}};
use super::assets::{load_binary, load_string};

pub async fn load_model_glb(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    transform_bind_group_layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<Vec<Object>> {
    let gltf_bin = load_binary(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_bin);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = gltf::Gltf::from_reader(gltf_reader)?;

    let buffer_data = extract_buffer_data(&gltf).await?;
    let materials = extract_materials(device, queue, texture_bind_group_layout, gltf.materials(), &buffer_data).await?;
    let mut objects = extract_objects(device, transform_bind_group_layout, file_name, gltf.scenes(), &buffer_data);

    // dbg!(&materials.iter().map(|m| m.name.clone()).collect::<Vec<_>>());

    let mut materials_per_idx = materials.into_iter().enumerate().fold(HashMap::new(), |mut acc, (idx, material)| {
        acc.insert(idx, material);
        acc
    });

    dbg!(&materials_per_idx.iter().map(|(idx, material)| (idx, material.name.clone())).collect::<Vec<_>>());

    objects.iter_mut().for_each(|object| {
        if let Some(object_3d) = &mut object.object_3d {
            for mesh in &mut object_3d.model.meshes {
                if let Some(material) = materials_per_idx.remove(&mesh.material) {
                    mesh.material = object_3d.model.materials.len();
                    object_3d.model.materials.push(material);
                } else {
                    panic!("no more material")
                }
            }

    //         let material_indexes = object_3d.model.meshes.iter().fold(HashSet::new(), |mut acc, mesh| {
    //             // println!("mesh {} need material {} and still having {} materials in the pocket", mesh.name, mesh.material, materials.len());
    //             acc.insert(mesh.material);
    //             acc
    //         });
    //         for material_idx in material_indexes {
    // // dbg!(&materials.iter().map(|m| m.name.clone()).collect::<Vec<_>>());
    //             if let Some(material) = materials_per_idx.remove(&material_idx) {
    //                 object_3d.model.materials.push(material);
    //             } else {
    //                 panic!("no more material");
    //             }
    //         }
    //         dbg!(&object_3d.model.materials.iter().map(|m| m.name.clone()).collect::<Vec<_>>());
    //         println!("current model have {} meshes", object_3d.model.meshes.len());
        }
    });

    println!("model have {} objects", objects.len());

    Ok(objects)
}

// pub async fn load_model_gltf(
//     file_name: &str,
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     texture_bind_group_layout: &wgpu::BindGroupLayout,
//     transform_bind_group_layout: &wgpu::BindGroupLayout,
// ) -> anyhow::Result<Object> {
//     let gltf_text = load_string(file_name).await?;
//     let gltf_cursor = Cursor::new(gltf_text);
//     let gltf_reader = BufReader::new(gltf_cursor);
//     let gltf = gltf::Gltf::from_reader(gltf_reader)?;

//     let buffer_data = extract_buffer_data(&gltf).await?;
//     let materials = extract_materials(device, queue, texture_bind_group_layout, gltf.materials(), &buffer_data).await?;
//     let meshes = extract_meshes(device, transform_bind_group_layout, file_name, gltf.scenes(), &buffer_data);

//     let model = Model { meshes, materials };
//     let mut object = Object::new();
//     object.set_object_3d(Object3D::new(device, model));
//     Ok(object)
// }

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

fn extract_objects(
    device: &wgpu::Device, 
    transform_bind_group_layout: &wgpu::BindGroupLayout,
    file_name: &str, 
    scenes: gltf::iter::Scenes,
    buffer_data: &Vec<Vec<u8>>
) -> Vec<Object> {

    let mut objects = Vec::<Object>::new();

    fn extract_from_node(
        node: &gltf::Node<'_>, 
        device: &wgpu::Device, 
        transform_bind_group_layout: &wgpu::BindGroupLayout, 
        objects: &mut Vec<Object>,
        buffer_data: &Vec<Vec<u8>>, 
        file_name: &str
    ) -> String {
        let mut object = Object::new();
        object.matrix = node.transform().matrix();
        if let Some(mesh) = node.mesh() {
            let mut meshes = Vec::<Mesh>::new();
            let primitives = mesh.primitives();
            primitives.for_each(|primitive| {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
                let mut vertices = Vec::new();

                // Extraire les positions
                let positions: Vec<[f32; 3]> = reader.read_positions()
                    .map(|positions| positions.collect())
                    .unwrap_or_default();

                // Extraire les coordonnées de texture
                let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0)
                    .map(|tex_coords| tex_coords.into_f32().collect())
                    .unwrap_or_default();

                // Extraire les normales
                let normals: Vec<[f32; 3]> = reader.read_normals()
                    .map(|normals| normals.collect())
                    .unwrap_or_default();

                // Construire les vertices
                for i in 0..positions.len() {
                    let position = positions.get(i).unwrap_or(&[0.0, 0.0, 0.0]);
                    let tex_coord = tex_coords.get(i).unwrap_or(&[0.0, 0.0]);
                    let normal = normals.get(i).unwrap_or(&[0.0, 0.0, 0.0]);

                    vertices.push(ModelVertex {
                        position: *position,
                        tex_coords: *tex_coord,
                        normal: *normal,
                    });
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }

                let report = &vertices.clone()[0..3];
                // dbg!(report);

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
                let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Transform Buffer"),
                    contents: bytemuck::cast_slice(&[TransformUniform::from(object.matrix_world)]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &transform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: transform_buffer.as_entire_binding(),
                    }],
                    label: None,
                });
                // dbg!(primitive.clone());
                let material = primitive.material();
                println!("{} have material {:?} at {:?}", file_name, material.name(), material.index());
                // dbg!(m);
                meshes.push(Mesh {
                    name: file_name.to_string(),
                    transform_bind_group,
                    transform_buffer,
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: primitive.material().index().unwrap_or(0),
                });
            });
            if meshes.len() > 0 {
                let model = Model { meshes, materials: Vec::new() };
                let object_3d = Object3D::new(device, model);
                object.set_object_3d(object_3d);
            }
        }
        let object_id = object.id.clone();
        objects.push(object);
        object_id
    }

    fn traverse(
        node: &gltf::Node<'_>,
        device: &wgpu::Device, 
        transform_bind_group_layout: &wgpu::BindGroupLayout,
        parent_id: Option<String>,
        objects: &mut Vec<Object>,
        buffer_data: &Vec<Vec<u8>>, 
        file_name: &str
    ) {
        let object_id = extract_from_node(
            node, 
            device, 
            transform_bind_group_layout, 
            objects, 
            buffer_data, 
            file_name
        );
        if let Some(parent_id) = parent_id {
            let (current_object, parent_object) = objects.iter_mut().fold((None, None), |mut acc, object| {
                if object.id == object_id {
                    acc.0 = Some(object);
                }
                else if object.id == parent_id {
                    acc.1 = Some(object);
                }
                acc
            });
            if let Some(current_object) = current_object {
                current_object.parent = Some(parent_id.clone());
            }
            if let Some(parent_object) = parent_object {
                parent_object.childrens.push(object_id.clone());
            }
        }
        for children in node.children() {
            traverse(&children, device, transform_bind_group_layout, Some(object_id.clone()), objects, buffer_data, file_name);
        }
    }

    for scene in scenes {
        for node in scene.nodes() {
            println!("node, but having {}", node.children().len());
            traverse(&node, device, transform_bind_group_layout,  None, &mut objects, buffer_data, file_name);
        }
    }

    objects
}

async fn extract_materials(
    device: &wgpu::Device, 
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    materials: gltf::iter::Materials<'_>,
    buffer_data: &Vec<Vec<u8>>
 ) -> anyhow::Result<Vec<Material>> {

    let mut extracted_materials = Vec::<Material>::new();

    for material in materials {
        println!("Looping thru materials {:#?}", material.name());
        let pbr = material.pbr_metallic_roughness();
        let texture_source = &pbr
            .base_color_texture()
            .map(|tex| {
                // dbg!(tex.clone());
                tex.texture().source().source()
            })
            .expect("texture");

        match texture_source {
            gltf::image::Source::View { view, mime_type } => {
                let buffer = &buffer_data[view.buffer().index()];
                let start = view.offset();
                let end = start + view.length();
                let image_data = &buffer[start..end];
                let material = load_material_from_bytes(
                    material.name().unwrap_or("Default Material"),
                    image_data,
                    device, 
                    queue, 
                    layout
                )?;
                extracted_materials.push(material);
            }
            gltf::image::Source::Uri { uri, mime_type } => {
                dbg!(mime_type);
                let material = load_material(uri, device, queue, layout).await?;
                extracted_materials.push(material);
            }
        };
    }
    let report = extracted_materials.iter().enumerate().map(|(idx, m)| (idx, m.name.clone())).collect::<Vec<_>>();
    dbg!(report);
    // println!("extracted materials {:#?}", Vec::clone(&extracted_materials).iter().map(|m, idx| (m.name)));
    Ok(extracted_materials)
}