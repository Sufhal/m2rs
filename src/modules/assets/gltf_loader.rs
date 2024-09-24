use std::{collections::HashMap, io::{BufReader, Cursor}};
use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use crate::modules::{assets::assets::{load_material, load_material_from_bytes}, core::{model::{BindGroupCreation, Material, Mesh, SimpleModel, SimpleVertex, SkinnedMeshVertex, SkinnedModel, TransformUniform, Transformable}, object::Object, object_3d::Object3D, skinning::{AnimationClip, Bone, BoneAnimation, Keyframes, Skeleton}}, pipelines::{simple_models_pipeline::SimpleModelPipeline, skinned_models_pipeline::SkinnedModelPipeline}};
use super::assets::load_binary;

pub async fn load_animation(path: &str, name: &str) -> anyhow::Result<AnimationClip> {
    let gltf_bin = load_binary(path).await?;
    let gltf_cursor = Cursor::new(gltf_bin);
    let gltf_reader = BufReader::new(gltf_cursor);
    let model = gltf::Gltf::from_reader(gltf_reader)?;
    let buffer_data = extract_buffer_data(&model).await?;
    let (_, bones_map, skin_joints_map) = extract_skeleton(&model, &buffer_data);
    let mut animations_clips = extract_animations(&model, &buffer_data, &bones_map, &skin_joints_map);
    let mut clip = animations_clips.swap_remove(0);
    clip.name = name.to_string();
    Ok(clip)
}

pub async fn load_model_glb(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    skinned_models_pipeline: &SkinnedModelPipeline,
    simple_models_pipeline: &SimpleModelPipeline,
) -> anyhow::Result<Vec<Object>> {
    let gltf_bin = load_binary(file_name).await?;
    let gltf_cursor = Cursor::new(gltf_bin);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = gltf::Gltf::from_reader(gltf_reader)?;

    let buffer_data = extract_buffer_data(&gltf).await?;
    let materials = extract_materials(device, queue, gltf.materials(), &buffer_data).await?;
    let mut objects = extract_objects(
        device, 
        skinned_models_pipeline,
        simple_models_pipeline,
        file_name, 
        &gltf, 
        &buffer_data
    );

    let mut materials_per_idx = materials.into_iter().enumerate().fold(HashMap::new(), |mut acc, (idx, material)| {
        acc.insert(idx, material);
        acc
    });

    objects.iter_mut().for_each(|object| {
        if let Some(object_3d) = &mut object.object3d {
            match object_3d {
                Object3D::Simple(simple) => {
                    for mesh in &mut simple.model.meshes {
                        if let Some(material) = materials_per_idx.remove(&mesh.material) {
                            mesh.material = simple.model.materials.len();
                            simple.model.materials.push(material);
                        } else {
                            panic!("no more material")
                        }
                    }
                    simple.model.meshes_bind_groups = simple.model.create_bind_groups(
                        &simple.model.meshes,
                        &simple.model.materials,
                        device, 
                        skinned_models_pipeline
                    );
                },
                Object3D::Skinned(skinned) => {
                    for mesh in &mut skinned.model.meshes {
                        if let Some(material) = materials_per_idx.remove(&mesh.material) {
                            mesh.material = skinned.model.materials.len();
                            skinned.model.materials.push(material);
                        } else {
                            panic!("no more material")
                        }
                    }
                    skinned.model.meshes_bind_groups = skinned.model.create_bind_groups(
                        &skinned.model.meshes,
                        &skinned.model.materials,
                        device, 
                        skinned_models_pipeline
                    );
                }
            }
                       
        }
    });

    Ok(objects)
}

pub async fn load_model_glb_with_name(
    file_name: &str,
    name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    skinned_models_pipeline: &SkinnedModelPipeline,
    simple_models_pipeline: &SimpleModelPipeline,
) -> anyhow::Result<Vec<Object>> {
    let mut objects = load_model_glb(file_name, device, queue, skinned_models_pipeline, simple_models_pipeline).await?;
    for object in &mut objects {
        object.name = Some(name.to_string());
    }
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
                .map(|(index, name, inverse_bind_matrix, translation, rotation, scale, childrens)| {
                    let parent_node = nodes.iter().find(|(_, _, _, _, _, _, childrens)| childrens.contains(index));
                    let parent_index = parent_node.map_or(None, |(parent_index, _, _, _, _, _, _)| Some(*bones_map.get(parent_index).unwrap()));
                    Bone::new(
                        parent_index, 
                        name.clone(),
                        *inverse_bind_matrix,
                        translation,
                        rotation,
                        scale,
                        childrens.clone(),
                    )
                })
                .collect::<Vec<_>>();

            let model_skeleton = Skeleton::new(bones);
            skeleton = Some(model_skeleton);
            break;
        }
    }

    // artifically creating bones for equip anchors
    for node in model.nodes() {
        if let Some(name) = node.name() {
            let index = node.index();
            if name.starts_with("equip_") {
                if let Some(skeleton) = &mut skeleton {
                    if let Some(parent_index) =  skeleton.bones.iter().position(|v| v.childrens.contains(&index)) {
                        let (translation, rotation, scale) = node.transform().decomposed();
                        skeleton.bones.push(
                            Bone::new(
                                Some(parent_index), 
                                Some(name.to_string()), 
                                Default::default(), 
                                &translation, 
                                &rotation, 
                                &scale, 
                                Vec::new()
                            )
                        );
                        match name {
                            "equip_left" | "equip_left_hand" => skeleton.equip_left = Some(skeleton.bones.len() - 1),
                            "equip_right" | "equip_right_hand" => skeleton.equip_right = Some(skeleton.bones.len() - 1),
                            _ => (),
                        };
                    }
                }
            }
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

    let _ = std::fs::write(
        std::path::Path::new("trash/last_animation_clips_loaded.txt"), 
        format!("{:#?}", animation_clips)
    );

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
    skinned_models_pipeline: &SkinnedModelPipeline,
    simple_models_pipeline: &SimpleModelPipeline,
    file_name: &str, 
    model: &gltf::Gltf,
    buffer_data: &Vec<Vec<u8>>
) -> Vec<Object> {

    let (skeleton, bones_map, skin_joints_map) = extract_skeleton(&model, &buffer_data);
    let animations_clips = extract_animations(&model, &buffer_data, &bones_map, &skin_joints_map);

    fn extract_from_node(
        node: &gltf::Node<'_>, 
        device: &wgpu::Device, 
        bones_map: &HashMap<usize, usize>,
        skin_joints_map: &HashMap<usize, usize>,
        buffer_data: &Vec<Vec<u8>>, 
        file_name: &str,
        model_meshes: &mut Vec<Mesh>,
        model_matrices: &mut Vec<Matrix4<f32>>,
    ) {
        if let Some(_) = bones_map.get(&node.index()) {
            // don't create Object from bone
            return;
        }

        let matrix = Matrix4::from(node.transform().matrix());
        model_matrices.push(matrix);

        if let Some(mesh) = node.mesh() {

            let primitives = mesh.primitives();
            
            primitives.for_each(|primitive| {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
                let mut vertices_simple: Vec<SimpleVertex> = Vec::new();
                let mut vertices_skinned: Vec<SkinnedMeshVertex> = Vec::new();

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

                let is_skinned = weights.len() > 0 && joints.len() > 0;
                
                for i in 0..positions.len() {

                    let position = positions.get(i).unwrap_or(&[0.0, 0.0, 0.0]);
                    let tex_coord = tex_coords.get(i).unwrap_or(&[0.0, 0.0]);
                    let normal = normals.get(i).unwrap_or(&[0.0, 0.0, 0.0]);

                    match is_skinned {
                        true => {
                            let weight = weights.get(i).unwrap_or(&[0.0, 0.0, 0.0, 0.0]);
                            let joint = joints.get(i).unwrap_or(&[0, 1, 2, 3]);
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
                            vertices_skinned.push(SkinnedMeshVertex::new(
                                *position,
                                *tex_coord,
                                *normal,
                                converted_joint,
                                *weight
                            ));
                        },
                        false => {
                            vertices_simple.push(SimpleVertex::new(
                                *position,
                                *tex_coord,
                                *normal,
                            ));
                        }
                    }

                }

                // only apply matrix on simple vertices
                // skinned vertices will inherit matrix from object matrix (because of skeleton)
                vertices_simple.apply_matrix(&matrix);

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: match is_skinned {
                        true => bytemuck::cast_slice(&vertices_skinned),
                        false => bytemuck::cast_slice(&vertices_simple)
                    },
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Transform Buffer"),
                    contents: bytemuck::cast_slice(&[TransformUniform::from(Default::default())]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                model_meshes.push(Mesh {
                    name: file_name.to_string(),
                    transform_buffer,
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: primitive.material().index().unwrap_or(0),
                });
            });
        }
    }

    fn traverse(
        node: &gltf::Node<'_>,
        device: &wgpu::Device, 
        bones_map: &HashMap<usize, usize>,
        skin_joints_map: &HashMap<usize, usize>,
        buffer_data: &Vec<Vec<u8>>, 
        file_name: &str,
        model_meshes: &mut Vec<Mesh>,
        model_matrices: &mut Vec<Matrix4<f32>>,
    ) {
        extract_from_node(
            node, 
            device, 
            bones_map,
            skin_joints_map,
            buffer_data, 
            file_name,
            model_meshes,
            model_matrices,
        );
        for children in node.children() {
            traverse(
                &children, 
                device, 
                bones_map, 
                skin_joints_map,
                buffer_data, 
                file_name,
                model_meshes,
                model_matrices,
            );
        }
    }

    let mut model_meshes = Vec::new();
    let mut model_matrices = Vec::new();

    for scene in model.scenes() {
        for node in scene.nodes() {
            traverse(
                &node, 
                device, 
                &bones_map, 
                &skin_joints_map,
                buffer_data, 
                file_name,
                &mut model_meshes,
                &mut model_matrices,
            );
        }
    }

    let mut object = Object::new();
    let object3d = match skeleton {
        Some(skeleton) => {
            object.matrix = model_matrices[0].into();
            let model = SkinnedModel { 
                meshes: model_meshes, 
                skeleton: skeleton.clone(), 
                animations: animations_clips.clone(),
                materials: Vec::new() ,
                meshes_bind_groups: Vec::new()
            };
            Object3D::from_skinned_model(device, &skinned_models_pipeline.bind_group_layouts, model)
        },
        None => {
            let model = SimpleModel {
                meshes: model_meshes,
                materials: Vec::new() ,
                meshes_bind_groups: Vec::new()
            };
            Object3D::from_simple_model(device, &simple_models_pipeline.bind_group_layouts, model)
        }
    };
    object.set_object_3d(object3d);

    vec![object]
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