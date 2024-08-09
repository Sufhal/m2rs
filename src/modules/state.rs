use std::path::Path;
use std::{fs, iter};
use cgmath::Rotation3;
use log::info;
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;
use winit::{
    event::*,
    keyboard::PhysicalKey,
    window::Window,
};
use crate::modules::core::object::{self, Object};
use crate::modules::core::object_3d::{self, Object3D};
use crate::modules::core::skinning::Keyframes;
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
use super::utils::performance_tracker::PerformanceTracker;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    new_render_pipeline: RenderPipeline,
    camera: camera::Camera,
    projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    pub mouse_pressed: bool,
    depth_texture: texture::Texture,
    pub window: &'a Window,
    scene: scene::Scene,
    performance_tracker: PerformanceTracker,
    time: std::time::Instant,
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

        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        
        let new_render_pipeline = RenderPipeline::new(
            &device, 
            &config, 
            Some(texture::Texture::DEPTH_FORMAT)
        );

        let mut scene = scene::Scene::new();

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

        Self {
            surface,
            device,
            queue,
            config,
            size,
            new_render_pipeline,
            camera,
            projection,
            camera_controller,
            mouse_pressed: false, 
            depth_texture,
            window,
            scene,
            performance_tracker: PerformanceTracker::new(),
            time: std::time::Instant::now()
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
            } => {
                match key {
                    KeyCode::KeyP => {
                        dbg!(self.performance_tracker.get_report());
                    },
                    _ => {},
                };
                self.camera_controller.process_keyboard(*key, *state)
            },
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
        self.performance_tracker.call_start("update");

        self.performance_tracker.call_start("update_camera");
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.new_render_pipeline.uniforms.camera.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.new_render_pipeline.buffers.camera,
            0,
            bytemuck::cast_slice(&[self.new_render_pipeline.uniforms.camera]),
        );
        self.performance_tracker.call_end("update_camera");

        self.performance_tracker.call_start("update_light");
        let old_position: cgmath::Vector3<_> = self.new_render_pipeline.uniforms.light.position.into();
        self.new_render_pipeline.uniforms.light.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into(); // UPDATED!
        self.queue.write_buffer(&self.new_render_pipeline.buffers.light, 0, bytemuck::cast_slice(&[self.new_render_pipeline.uniforms.light]));
        self.performance_tracker.call_end("update_light");

        // self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        // self.queue.write_buffer(
        //     &self.camera_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera_uniform]),
        // );
        // let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        // self.light_uniform.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into(); // UPDATED!
        // self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
        
        self.performance_tracker.call_start("update_objects");

        let elapsed = self.time.elapsed().as_secs_f64();

        for object in self.scene.get_all_objects() {
            if let Some(object_3d) = &mut object.object_3d {
                let model = &mut object_3d.model;
                if let Some(animations) = &model.animations {
                    // let _ = fs::write(Path::new("trash/debug.txt"), format!("{:#?}", &animations));
                    // panic!();
                    if let Some(skeleton) = &mut model.skeleton {
                        if let Some(animation) = animations.iter().find(|clip| &clip.name == "Run") {
                            for bone_animation in &animation.animations {
                                match &bone_animation.keyframes {
                                    Keyframes::Translation(frames) => {
                                        if frames.len() == 0 {
                                            return;
                                        }
                                        else if frames.len() == 1 {
                                            let frame = &frames[0];
                                            skeleton.bones[bone_animation.bone].set_translation(&[frame[0], frame[1], frame[2]]);
                                        }
                                        else {

                                        }
                                        let frame = &frames[0];
                                        // println!("{frame:?}");
                                        skeleton.bones[bone_animation.bone].set_translation(&[frame[0] * 1000.0, frame[1] * 1000.0, frame[2] * 1000.0]);
                                        // skeleton.bones[bone_animation.bone].set_translation(&[frame[0] * 1000.0, frame[1] * 1000.0, frame[2] * 1000.0]);
                                    },
                                    Keyframes::Other => {},
                                };
                            }
                        }
                        skeleton.calculate_world_matrices();
                        object_3d.update_skeleton(&self.queue);
                        // skeleton.bones
                    }
                    // panic!()
                }

                // for (index, instance) in object_3d.get_instances().iter_mut().enumerate() {
                //     // let rotation_speed = std::f32::consts::PI * 2.0; // 90 degrés par seconde
                //     // let rotation_angle = rotation_speed * dt.as_secs_f32();
                //     // let rotation = rotation_angle * index as f32 * 0.2;
                //     // instance.add_xyz_rotation(rotation, rotation, rotation);
                // }
            }
        }
        self.performance_tracker.call_end("update_objects");

        self.performance_tracker.call_start("update_scene");
        self.scene.compute_world_matrices();
        self.scene.update_objects_buffers(&self.queue);
        self.performance_tracker.call_end("update_scene");

        self.performance_tracker.call_end("update");
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.performance_tracker.call_start("render");
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

            self.performance_tracker.call_start("render_draw_scene");
            render_pass.draw_scene(
                &self.queue,
                &mut self.scene, 
                &self.new_render_pipeline
            );
            self.performance_tracker.call_end("render_draw_scene");
        }

        self.queue.submit(iter::once(encoder.finish()));
        self.performance_tracker.call_start("render_present");
        output.present();
        self.performance_tracker.call_end("render_present");
        self.performance_tracker.call_end("render");
        Ok(())
    }
}