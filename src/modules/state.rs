use std::path::Path;
use std::{fs, iter};
use cgmath::Rotation3;
use log::info;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    keyboard::PhysicalKey,
    window::Window,
};
use crate::modules::core::object::{self, Object};
use crate::modules::core::object_3d::{self, Object3D};
use crate::modules::core::{instance, light, model, texture};
use crate::modules::assets::assets;
use crate::modules::camera::camera;
use crate::modules::geometry::sphere::Sphere;
use crate::modules::geometry::buffer::ToMesh;
use model::Vertex;
use super::assets::gltf_loader::load_model_glb;
use super::core::model::Model;
use super::core::object_3d::Transform;
use super::core::scene;
use super::geometry::plane::Plane;
use super::pipelines::render_pipeline::RenderPipeline;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // render_pipeline: wgpu::RenderPipeline,
    new_render_pipeline: RenderPipeline,
    camera: camera::Camera,
    projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    pub mouse_pressed: bool,
    // camera_uniform: camera::CameraUniform,
    // camera_buffer: wgpu::Buffer,
    // camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    // light_uniform: light::LightUniform,
    // light_buffer: wgpu::Buffer,
    // light_bind_group: wgpu::BindGroup,
    // transform_bind_group_layout: wgpu::BindGroupLayout,
    pub window: &'a Window,
    scene: scene::Scene
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            // backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        // wgpu::Limits::downlevel_webgl2_defaults()
                        wgpu::Limits::default()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format.remove_srgb_suffix(),
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.add_srgb_suffix()],
            desired_maximum_frame_latency: 2,
        };

        // let texture_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Texture {
        //                     multisampled: false,
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //                 count: None,
        //             },
        //         ],
        //         label: Some("texture_bind_group_layout"),
        //     });

        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // let camera_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //         label: Some("camera_bind_group_layout"),
        //     });

        // let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &camera_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: camera_buffer.as_entire_binding(),
        //     }],
        //     label: Some("camera_bind_group"),
        // });

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let light_uniform = light::LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };
        
         // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // let light_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //         label: None,
        //     });

        // let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &light_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: light_buffer.as_entire_binding(),
        //     }],
        //     label: None,
        // });

        // TRANSFORM

        // let transform_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //         label: Some("transform_bind_group_layout"),
        //     });

        //  SKINNING

        // let bones_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Storage { read_only: true },
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //         label: Some("bones_bind_group_layout"),
        //     });

        // let render_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("Render Pipeline Layout"),
        //         bind_group_layouts: &[
        //             &texture_bind_group_layout, 
        //             &camera_bind_group_layout,
        //             &light_bind_group_layout,
        //             &transform_bind_group_layout,
        //             // &bones_bind_group_layout,
        //         ],
        //         push_constant_ranges: &[],
        //     });

        // let render_pipeline = {
        //     let shader = wgpu::ShaderModuleDescriptor {
        //         label: Some("Normal Shader"),
        //         source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        //     };
        //     functions::create_render_pipeline(
        //         &device,
        //         &render_pipeline_layout,
        //         config.format,
        //         Some(texture::Texture::DEPTH_FORMAT),
        //         &[
        //             model::ModelVertex::desc(), 
        //             instance::InstanceRaw::desc()
        //         ],
        //         shader,
        //         &config
        //     )
        // };

        let new_render_pipeline = RenderPipeline::new(
            &device, 
            &config, 
            Some(texture::Texture::DEPTH_FORMAT)
        );

        let mut scene = scene::Scene::new();

        // let mut cube =
        //     assets::load_model("cube.obj", &device, &queue, &texture_bind_group_layout, &transform_bind_group_layout)
        //         .await
        //         .unwrap();

        // if let Some(object_3d) = cube.get_object_3d() {
        //     for i in 2..11 {
        //         let instance = object_3d.request_instance(&device);
        //         instance.take();
        //         instance.set_position(cgmath::Vector3 { x: (i * 3) as f32, y: 0.0, z: 0.0 });
        //     }
        // }
        // scene.add(cube);

        // let plane = Plane::new(10.0, 10.0, 1, 1);
        // let mesh = plane.to_mesh(&device, &transform_bind_group_layout, "A plane".to_string());
        // let material = assets::load_material("test.png", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let model = Model { 
        //     meshes: vec![mesh], 
        //     skeleton: None,
        //     materials: vec![material] 
        // };
        // let mut object = Object::new();
        // object.set_object_3d(Object3D::new(&device, model));
        // let instance = object.get_object_3d().unwrap().request_instance(&device);
        // instance.add_y_position(-1.0);
        // instance.take();
        // scene.add(object);

        // let sphere = Sphere::new(2.0, 16, 16);
        // let mesh = sphere.to_mesh(&device, &transform_bind_group_layout, "A sphere".to_string());
        // let material = assets::load_material("test.png", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let model = Model { 
        //     meshes: vec![mesh], 
        //     skeleton: None,
        //     materials: vec![material] 
        // };
        // let mut object = Object::new();
        // object.set_object_3d(Object3D::new(&device, model));
        // let instance = object.get_object_3d().unwrap().request_instance(&device);
        // instance.add_y_position(1.0);
        // instance.take();
        // scene.add(object);
        
        // let material = assets::load_material("test.png", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let model_objects = load_model_glb("vladimir.glb", &device, &queue, &texture_bind_group_layout, &transform_bind_group_layout).await.expect("unable to load");
        // for mut object in model_objects {
        //     let id = object.id.clone();
        //     if let Some(object_3d) = &mut object.object_3d {
        //         // dbg!(&object.matrix);
        //         // println!("object {} have mesh", id);
        //         let instance = object_3d.request_instance(&device);
        //         instance.add_x_position(-1.0);
        //         instance.take();
        //         // if let Some(skeleton) = &object_3d.model.skeleton {
        //         //     let sphere = Sphere::new(2.0, 16, 16);
        //         //     let mesh = sphere.to_mesh(&device, &transform_bind_group_layout, "A sphere".to_string());
        //         //     let material = assets::load_material("test.png", &device, &queue, &texture_bind_group_layout).await.unwrap();
        //         //     let model = Model { 
        //         //         meshes: vec![mesh], 
        //         //         skeleton: None,
        //         //         materials: vec![material] 
        //         //     };
        //         //     let mut object = Object::new();
        //         //     object.set_object_3d(Object3D::new(&device, model));
        //         //     for bone in &skeleton.bones {
        //         //         let instance = object.get_object_3d().unwrap().request_instance(&device);
        //         //         let matrix = cgmath::Matrix4::from(bone.transform_matrix);
        //         //         instance.add_xyz_position(matrix.w[0], matrix.w[1], matrix.w[2]);
        //         //         instance.take();
        //         //     }
        //         //     scene.add(object);
        //         // }
        //     }
        //     scene.add(object);
        // }

        let model_objects = load_model_glb(
            // "vladimir.glb", 
            "shaman.glb", 
            &device, 
            &queue, 
            &new_render_pipeline
        ).await.expect("unable to load");
        dbg!(model_objects.len());
        // dbg!(model_objects.iter().map(|o| &o.name).collect::<Vec<_>>());
        for mut object in model_objects {
            let id = object.id.clone();
            if let Some(object_3d) = &mut object.object_3d {
                // dbg!(&object.matrix);
                // println!("object {} have mesh", id);
                let instance = object_3d.request_instance(&device);
                instance.add_x_position(1.0);
                instance.take();
                
            }
            scene.add(object);
        }

        let model_objects = load_model_glb(
            "vladimir.glb", 
            // "shaman.glb", 
            &device, 
            &queue, 
            &new_render_pipeline
        ).await.expect("unable to load");
        dbg!(model_objects.len());
        // dbg!(model_objects.iter().map(|o| &o.name).collect::<Vec<_>>());
        for mut object in model_objects {
            let id = object.id.clone();
            if let Some(object_3d) = &mut object.object_3d {
                // dbg!(&object.matrix);
                // println!("object {} have mesh", id);
                let instance = object_3d.request_instance(&device);
                instance.add_x_position(-1.0);
                instance.take();
                
            }
            scene.add(object);
        }


        // let model_objects = load_model_glb("official_gltf/gltf_binary/2CylinderEngine.glb", &device, &queue, &texture_bind_group_layout, &transform_bind_group_layout).await.expect("unable to load");
        // for mut object in model_objects {
        //     let id = object.id.clone();
        //     if let Some(object_3d) = &mut object.object_3d {
        //         // dbg!(&object.matrix);
        //         // println!("object {} have mesh", id);
        //         let instance = object_3d.request_instance(&device);
        //         instance.add_x_position(0.0);
        //         instance.take();
        //     }
        //     scene.add(object);
        // }

        scene.compute_world_matrices();
        scene.update_objects_buffers(&queue);


        // let matrix_world_of_root_bone = scene.get_all_objects().iter().find(|obj| obj.name == Some("RootBone".to_string())).expect("find RootBone");

    
        // let sphere = Sphere::new(0.25, 16, 16);
        // let mesh = sphere.to_mesh(&device, &transform_bind_group_layout, "A sphere".to_string());
        // let material = assets::load_material("test.png", &device, &queue, &texture_bind_group_layout).await.unwrap();
        // let model = Model { 
        //     meshes: vec![mesh], 
        //     skeleton: None,
        //     materials: vec![material] 
        // };
        // let mut object = Object::new();
        // object.set_object_3d(Object3D::new(&device, model));

        // for bone in &skeleton.bones {
        //     let instance = object.get_object_3d().unwrap().request_instance(&device);
        //     let matrix = cgmath::Matrix4::from(bone.bind_matrix) * cgmath::Matrix4::from_scale(0.01);
        //     instance.add_xyz_position(matrix.w[0], matrix.w[1], matrix.w[2]);
        //     instance.take();
        // }

        // scene.add(object);


        Self {
            surface,
            device,
            queue,
            config,
            size,
            // render_pipeline,
            new_render_pipeline,
            camera,
            projection,
            camera_controller,
            mouse_pressed: false, 
            // camera_buffer,
            // camera_bind_group,
            // camera_uniform,
            depth_texture,
            // light_buffer,
            // light_uniform,
            // light_bind_group,
            // transform_bind_group_layout,
            window,
            scene,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            info!("new size {new_size:#?}");
            self.size = new_size;
            self.config.width = std::cmp::min(new_size.width, wgpu::Limits::default().max_texture_dimension_2d);
            self.config.height = std::cmp::min(new_size.height, wgpu::Limits::default().max_texture_dimension_2d);
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.projection.resize(new_size.width, new_size.height);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.new_render_pipeline.uniforms.camera.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.new_render_pipeline.buffers.camera,
            0,
            bytemuck::cast_slice(&[self.new_render_pipeline.uniforms.camera]),
        );

        let old_position: cgmath::Vector3<_> = self.new_render_pipeline.uniforms.light.position.into();
        self.new_render_pipeline.uniforms.light.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into(); // UPDATED!
        self.queue.write_buffer(&self.new_render_pipeline.buffers.light, 0, bytemuck::cast_slice(&[self.new_render_pipeline.uniforms.light]));

        // self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        // self.queue.write_buffer(
        //     &self.camera_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera_uniform]),
        // );
        // let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        // self.light_uniform.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into(); // UPDATED!
        // self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
        
        
        for object in self.scene.get_all_objects() {
            if let Some(object_3d) = object.get_object_3d() {
                let model = &object_3d.model;
                if let Some(animations) = &model.animations {
                    fs::write(Path::new("trash/debug.txt"), format!("{:#?}", dbg!(&animations)));
                    panic!()
                }

                for (index, instance) in object_3d.get_instances().iter_mut().enumerate() {
                    // let rotation_speed = std::f32::consts::PI * 2.0; // 90 degrés par seconde
                    // let rotation_angle = rotation_speed * dt.as_secs_f32();
                    // let rotation = rotation_angle * index as f32 * 0.2;
                    // instance.add_xyz_rotation(rotation, rotation, rotation);
                }
            }
        }
        self.scene.compute_world_matrices();
        self.scene.update_objects_buffers(&self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            use scene::DrawScene;
            render_pass.set_pipeline(&self.new_render_pipeline.pipeline);
            render_pass.draw_scene(
                &self.queue,
                &mut self.scene, 
                &self.new_render_pipeline
            );
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}