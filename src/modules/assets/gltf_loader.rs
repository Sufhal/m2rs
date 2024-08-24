use std::{collections::HashMap, io::{BufReader, Cursor}};
use cgmath::Matrix4;
use wgpu::util::DeviceExt;

use crate::modules::{assets::assets::{load_material, load_material_from_bytes}, core::{model::{Material, Mesh, Model, ModelVertex, TransformUniform}, object::Object, object_3d::Object3D, skinning::{AnimationClip, Bone, BoneAnimation, Keyframes, Skeleton}}, pipelines::render_pipeline::{RenderBindGroupLayouts, RenderPipeline}};
use super::assets::load_binary;

pub async fn load_animations(
    file_name: &str,
    skeleton: &Skeleton
) -> anyhow::Result<Vec<AnimationClip>> {
    let gltf_bin = load_binary(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_bin);
    let gltf_reader = BufReader::new(gltf_cursor);
    let model = gltf::Gltf::from_reader(gltf_reader)?;
    let buffer_data = extract_buffer_data(&model).await?;
    let (attached_skeleton, bones_map, skin_joints_map) = extract_skeleton(&model, &buffer_data);
    let mut animations_clips = extract_animations(&model, &buffer_data, &bones_map, &skin_joints_map);

    let _ = std::fs::write(
        std::path::Path::new(&format!("trash/skeleton_original_{file_name}.txt")), 
        format!("{:#?}", skeleton)
    );

    // let _ = std::fs::write(
    //     std::path::Path::new(&format!("trash/skeleton_from_animation_{file_name}.txt")), 
    //     format!("{:#?}", &attached_skeleton.clone().unwrap())
    // );

    if let Some(attached_skeleton) = attached_skeleton {

        let mut map_attached_to_current = HashMap::new();
        for (index, bone) in skeleton.bones.iter().enumerate() {
            match attached_skeleton.bones.iter().position(|v| v.name == bone.name) {
                Some(position) => {
                    // println!("Bone name is '{:?}' at position {index}, in attached skeleton, found at position {position}", attached_skeleton.bones[position].name);
                    map_attached_to_current.insert(position, index);
                },
                None => {}
            };
        }

        let _ = std::fs::write(
            std::path::Path::new(&format!("trash/clips_original_{file_name}.txt")), 
            format!("{:#?}", &animations_clips)
        );
    
        for clip in &mut animations_clips {
            for animation in &mut clip.animations {
                let index = *map_attached_to_current.get(&animation.bone).unwrap();
                // println!("for animation of bone {}, the index {index} will be used using map_attached_to_current", animation.bone);
                animation.bone = index;
            }
        }

        // dbg!(&map_attached_to_current);

        let _ = std::fs::write(
            std::path::Path::new(&format!("trash/clips_modified_{file_name}.txt")), 
            format!("{:#?}", &animations_clips)
        );

    }
    

    let _ = std::fs::write(
        std::path::Path::new(&format!("trash/animations_{file_name}.txt")), 
        format!("{:#?}", &animations_clips)
    );

    let _ = std::fs::write(
        std::path::Path::new(&format!("trash/skeleton_{file_name}.txt")), 
        format!("{:#?}", &skeleton)
    );

    Ok(animations_clips)
} 

pub async fn load_model_glb(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    render_pipeline: &RenderPipeline,
) -> anyhow::Result<Vec<Object>> {
    let gltf_bin = load_binary(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_bin);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = gltf::Gltf::from_reader(gltf_reader)?;

    let buffer_data = extract_buffer_data(&gltf).await?;
    let materials = extract_materials(device, queue, gltf.materials(), &buffer_data).await?;
    let mut objects = extract_objects(
        device, 
        &render_pipeline.bind_group_layouts, 
        file_name, 
        &gltf, 
        &buffer_data
    );

    let mut materials_per_idx = materials.into_iter().enumerate().fold(HashMap::new(), |mut acc, (idx, material)| {
        acc.insert(idx, material);
        acc
    });

    // dbg!(&materials_per_idx.iter().map(|(idx, material)| (idx, material.name.clone())).collect::<Vec<_>>());

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
            object_3d.model.create_bind_groups(device, render_pipeline);
        }
    });

    Ok(objects)
}

fn extract_skeleton(
    model: &gltf::Gltf,
    buffer_data: &Vec<Vec<u8>>
) -> (Option<Skeleton>, HashMap<usize, usize>, HashMap<usize, usize>) {
    let mut skeleton = None;
    let mut bones_map = HashMap::new();
    let mut skin_joints_map = HashMap::new();

    // looking for bones
    for skin in model.skins() {

        let mut nodes = Vec::new();

        let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
        let inverse_bind_matrices: Vec<Matrix4<f32>> = reader
            .read_inverse_bind_matrices()
            .expect("No inverse bind matrices")
            .map(|m| Matrix4::from(m))
            .collect();

        // Create a mapping from joint index to its inverse bind matrix
        let joint_map: std::collections::HashMap<usize, [[f32; 4]; 4]> = skin
            .joints()
            .enumerate()
            .map(|(i, joint)| (joint.index(), inverse_bind_matrices[i].into()))
            .collect();

        // let _ = std::fs::write(std::path::Path::new(&format!("trash/inverse_bind_matrices_{file_name}.txt")), format!("{:#?}", &joint_map));
        
        for (joint_idx, joint) in skin.joints().enumerate() {
            let index = joint.index();
            let name = joint.name().map_or(None, |str| Some(str.to_string()));
            bones_map.insert(index, nodes.len());
            skin_joints_map.insert(joint_idx, nodes.len());
            let (translation, rotation, scale) = joint.transform().decomposed();
            nodes.push((
                index, 
                name,
                *joint_map.get(&index).unwrap(),
                translation,
                rotation,
                scale,
                joint.children().map(|children| children.index()).collect::<Vec<_>>()
            ));
        }
        if nodes.len() > 0 {
            let bones = nodes
                .iter()
                .map(|(index, name, inverse_bind_matrix, translation, rotation, scale, _)| {
                    let parent_node = nodes.iter().find(|(_, _, _, _, _, _, childrens)| childrens.contains(index));
                    Bone::new(
                        parent_node.map_or(None, |(parent_index, _, _, _, _, _, _)| Some(*bones_map.get(parent_index).unwrap())), 
                        // (OPENGL_TO_WGPU_MATRIX * cgmath::Matrix4::from(*matrix)).into()
                        name.clone(),
                        *inverse_bind_matrix,
                        translation,
                        rotation,
                        scale
                    )
                })
                .collect::<Vec<_>>();

            let mut model_skeleton = Skeleton { bones };
            skeleton = Some(model_skeleton);
            break;
        }
    }

    (skeleton, bones_map, skin_joints_map)
}

fn extract_animations(
    model: &gltf::Gltf,
    buffer_data: &Vec<Vec<u8>>,
    bones_map: &HashMap<usize, usize>,
    _skin_joints_map: &HashMap<usize, usize>,
) -> Vec<AnimationClip> {

    let mut animation_clips = Vec::new();

    for animation in model.animations() {
        
        let name = animation.name().unwrap_or("Default").to_string();
        let mut duration = 0.0;
        let mut animations = Vec::new();

        for channel in animation.channels() {
            // dbg!(&bones_map);
            // println!("looking for bone original index {}", channel.target().node().index());

            let target_node = channel.target().node();

            if let Some(bone) = bones_map.get(&target_node.index()) {
                let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
                let timestamps = if let Some(inputs) = reader.read_inputs() {
                    match inputs {
                        gltf::accessor::Iter::Standard(times) => {
                            let times: Vec<f32> = times.collect();
                            let times = times.iter().map(|v| *v as f64).collect::<Vec<_>>();
                            if let Some(time) = times.last() {
                                if *time > duration {
                                    duration = *time;
                                }
                            }
                            times
                        }
                        gltf::accessor::Iter::Sparse(_) => {
                            println!("Sparse keyframes not supported");
                            let times: Vec<f64> = Vec::new();
                            times
                        }
                    }
                } else {
                    println!("We got problems");
                    let times: Vec<f64> = Vec::new();
                    times
                };

                let keyframes = if let Some(outputs) = reader.read_outputs() {
                    match outputs {
                        gltf::animation::util::ReadOutputs::Translations(translation) => {
                            Keyframes::Translation(translation.collect())
                        },
                        gltf::animation::util::ReadOutputs::Rotations(rotation) => {
                            Keyframes::Rotation(rotation.into_f32().collect())
                        },
                        gltf::animation::util::ReadOutputs::Scales(scale) => {
                            Keyframes::Scale(scale.collect())
                        }
                        _other => {
                            Keyframes::Other
                        }
                    }
                } else {
                    println!("We got problems");
                    Keyframes::Other
                };

                animations.push(
                    BoneAnimation {
                        bone: *bone,
                        keyframes,
                        timestamps,
                    }
                );
            }
            else {
                // dbg!("target_node name {:?}", target_node.name());
                // panic!();
            }
        }

        animation_clips.push(
            AnimationClip {
                name,
                duration,
                animations
            }
        );
    }

    animation_clips
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

fn extract_objects(
    device: &wgpu::Device, 
    bind_group_layouts: &RenderBindGroupLayouts,
    file_name: &str, 
    model: &gltf::Gltf,
    buffer_data: &Vec<Vec<u8>>
) -> Vec<Object> {

    let mut objects = Vec::new();
    let (skeleton, bones_map, skin_joints_map) = extract_skeleton(&model, &buffer_data);
    let animations_clips = extract_animations(&model, &buffer_data, &bones_map, &skin_joints_map);

    fn extract_from_node(
        node: &gltf::Node<'_>, 
        device: &wgpu::Device, 
        bind_group_layouts: &RenderBindGroupLayouts,
        objects: &mut Vec<Object>,
        bones_map: &HashMap<usize, usize>,
        skin_joints_map: &HashMap<usize, usize>,
        skeleton: &Option<Skeleton>,
        animation_clips: &Vec<AnimationClip>,
        buffer_data: &Vec<Vec<u8>>, 
        file_name: &str
    ) -> Option<String> {
        if let Some(_) = bones_map.get(&node.index()) {
            return None;
        }
        let object = Object::new();
        let position = objects.len();
        objects.push(object);
        let object = objects.get_mut(position).unwrap();
        let object_id = object.id.clone();

        if let Some(name) = node.name() {
            object.name = Some(name.to_string());
        }
        // let (translation, rotation, scale) = node.transform().decomposed();
        // object.matrix = (Matrix4::from_translation(translation.into()) * Matrix4::from(Quaternion::from([rotation[3], rotation[0], rotation[1], rotation[2]])) * Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2])).into();
        object.matrix = node.transform().matrix();
        if let Some(mesh) = node.mesh() {
            let mut meshes = Vec::new();
            let primitives = mesh.primitives();
            primitives.for_each(|primitive| {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
                let mut vertices = Vec::new();

                let positions: Vec<[f32; 3]> = reader.read_positions()
                    .map(|positions| positions.collect())
                    .unwrap_or_default();

                let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0)
                    .map(|tex_coords| tex_coords.into_f32().collect())
                    .unwrap_or_default();

                let normals: Vec<[f32; 3]> = reader.read_normals()
                    .map(|normals| normals.collect())
                    .unwrap_or_default();

                let weights: Vec<[f32; 4]> = reader.read_weights(0)
                    .map(|weights| weights.into_f32().collect())
                    .unwrap_or_default();

                let joints: Vec<[u16; 4]> = reader.read_joints(0)
                    .map(|joints| joints.into_u16().collect())
                    .unwrap_or_default();
                    
                

                for i in 0..positions.len() {
                    let position = positions.get(i).unwrap_or(&[0.0, 0.0, 0.0]);
                    let tex_coord = tex_coords.get(i).unwrap_or(&[0.0, 0.0]);
                    let normal = normals.get(i).unwrap_or(&[0.0, 0.0, 0.0]);
                    let weight = weights.get(i).unwrap_or(&[0.0, 0.0, 0.0, 0.0]);
                    let joint = joints.get(i).unwrap_or(&[0, 1, 2, 3]);
                    // let converted_joint: [u32; 4] = core::array::from_fn(|i| *(joints[i] as u32));
                    let converted_joint: [u32; 4] = core::array::from_fn(|i| {
                        match skin_joints_map.get(&(joint[i] as usize)) {
                            Some(index) => {
                                *index as u32
                            },
                            None => {
                                panic!()
                            }
                        }
                    });
                    vertices.push(ModelVertex::new(
                        *position,
                        *tex_coord,
                        *normal,
                        converted_joint,
                        *weight
                    ));
                }

                // dbg!(&skeleton.as_ref().unwrap().bones[6].name);
                // dbg!(&bones_map);

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }

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
                meshes.push(Mesh {
                    name: file_name.to_string(),
                    transform_buffer,
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: primitive.material().index().unwrap_or(0),
                });
            });
            if meshes.len() > 0 {
                let model = Model { 
                    meshes, 
                    skeleton: skeleton.clone().unwrap(), 
                    animations: animation_clips.clone(),
                    materials: Vec::new() ,
                    meshes_bind_groups: Vec::new()
                };
                let object_3d = Object3D::new(device, bind_group_layouts, model);
                object.set_object_3d(object_3d);
            }
        }
        Some(object_id)
    }

    fn traverse(
        node: &gltf::Node<'_>,
        device: &wgpu::Device, 
        bind_group_layouts: &RenderBindGroupLayouts,
        parent_id: Option<String>,
        objects: &mut Vec<Object>,
        bones_map: &HashMap<usize, usize>,
        skin_joints_map: &HashMap<usize, usize>,
        skeleton: &Option<Skeleton>,
        animation_clips: &Vec<AnimationClip>,
        buffer_data: &Vec<Vec<u8>>, 
        file_name: &str
    ) {
        let object_id = extract_from_node(
            node, 
            device, 
            bind_group_layouts, 
            objects, 
            bones_map,
            skin_joints_map,
            skeleton,
            animation_clips,
            buffer_data, 
            file_name
        );
        if let Some(object_id) = object_id {
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
                traverse(
                    &children, 
                    device, 
                    bind_group_layouts, 
                    Some(object_id.clone()), 
                    objects, 
                    bones_map, 
                    skin_joints_map,
                    skeleton, 
                    animation_clips,
                    buffer_data, 
                    file_name
                );
            }
        }
    }

    for scene in model.scenes() {
        for node in scene.nodes() {
            traverse(
                &node, 
                device, 
                bind_group_layouts, 
                None, 
                &mut objects, 
                &bones_map, 
                &skin_joints_map,
                &skeleton, 
                &animations_clips,
                buffer_data, 
                file_name
            );
        }
    }

    objects
}

async fn extract_materials(
    device: &wgpu::Device, 
    queue: &wgpu::Queue,
    materials: gltf::iter::Materials<'_>,
    buffer_data: &Vec<Vec<u8>>
 ) -> anyhow::Result<Vec<Material>> {

    let mut extracted_materials = Vec::<Material>::new();

    for material in materials {
        let pbr = material.pbr_metallic_roughness();
        let texture_source = &pbr
            .base_color_texture()
            .map(|tex| {
                // dbg!(tex.clone());
                tex.texture().source().source()
            })
            .expect("texture");

        match texture_source {
            gltf::image::Source::View { view, mime_type: _ } => {
                let buffer = &buffer_data[view.buffer().index()];
                let start = view.offset();
                let end = start + view.length();
                let image_data = &buffer[start..end];
                let material = load_material_from_bytes(
                    material.name().unwrap_or("Default Material"),
                    image_data,
                    device, 
                    queue,
                )?;
                extracted_materials.push(material);
            }
            gltf::image::Source::Uri { uri, mime_type } => {
                dbg!(mime_type);
                let material = load_material(uri, device, queue).await?;
                extracted_materials.push(material);
            }
        };
    }
    Ok(extracted_materials)
}