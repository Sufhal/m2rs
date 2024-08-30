use std::path::Path;
use std::{fs, iter};
use cgmath::Rotation3;
use log::info;
use winit::keyboard::KeyCode;
use winit::{
    event::*,
    keyboard::PhysicalKey,
    window::Window,
};
use crate::modules::core::model::DrawCustomMesh;
use crate::modules::core::texture;
use crate::modules::camera::camera;
use super::assets::gltf_loader::load_model_glb;
use super::character::character::{Character, CharacterKind, NPCType};
use super::core::object_3d::{Transform, TranslateWithScene};
use super::core::scene;
use super::pipelines::common_pipeline::CommonPipeline;
use super::pipelines::render_pipeline::RenderPipeline;
use super::pipelines::terrain_pipeline::TerrainPipeline;
use super::pipelines::water_pipeline::WaterPipeline;
use super::terrain::terrain::Terrain;
use super::utils::time_factory::{Instant, TimeFactory};
// use super::utils::performance_tracker::PerformanceTracker;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub common_pipeline: CommonPipeline,
    pub new_render_pipeline: RenderPipeline,
    pub terrain_pipeline: TerrainPipeline,
    pub water_pipeline: WaterPipeline,
    camera: camera::Camera,
    projection: camera::Projection,
    pub camera_controller: camera::CameraController,
    pub mouse_pressed: bool,
    depth_texture: texture::Texture,
    pub window: &'a Window,
    pub scene: scene::Scene,
    // performance_tracker: PerformanceTracker,
    // time: std::time::Instant,
    instant: Instant,
    time_factory: TimeFactory,
    pub characters: Vec<Character>,
    pub terrains: Vec<Terrain>,
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

        let camera = camera::Camera::new((515.0, 5.0, 643.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        // let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        
        let common_pipeline = CommonPipeline::new(&device);
        let terrain_pipeline = TerrainPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &common_pipeline);
        let water_pipeline = WaterPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &common_pipeline);
        let new_render_pipeline = RenderPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &common_pipeline);

        let mut scene = scene::Scene::new();

        // dbg!(&shaman_animations);

        let model_objects = load_model_glb(
            "pack/pc/shaman_m/shaman_cheonryun.glb", 
            // "fox.glb", 
            &device, 
            &queue, 
            &new_render_pipeline
        ).await.expect("unable to load");
        for mut object in model_objects {
            if let Some(object_3d) = &mut object.object_3d {
                // let clips = load_animations(
                //     "shaman_wait_1.glb", 
                //     // "shaman_cheonryun.glb", 
                //     // "fox.glb", 
                //     &object_3d.model.skeleton
                // ).await.unwrap();
                // object_3d.set_animations(clips);
                // object_3d.model.animations = clips;
                
  
                // dbg!(&object.matrix);
                // println!("object {} have mesh", id);
                for i in 0..10 {
                    let instance = object_3d.request_instance(&device);
                    instance.add_x_position(0.5 + (i as f32 / 2.0));
                    instance.take();
                    // dbg!(&instance.id);
                }
                
            }
            scene.add(object);
        }

        // let shaman_animations = load_animations("run.glb").await.unwrap();
        // dbg!(&shaman_animations);


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

        let _ = fs::write(Path::new("trash/scene_objects.txt"), format!("{:#?}", &&scene.get_all_objects().iter().map(|object| (object.name.clone(), object.matrix)).collect::<Vec<_>>()));


        let mut state = Self {
            surface,
            device,
            queue,
            config,
            size,
            common_pipeline,
            new_render_pipeline,
            terrain_pipeline,
            water_pipeline,
            camera,
            projection,
            camera_controller,
            mouse_pressed: false, 
            depth_texture,
            window,
            scene,
            // performance_tracker: PerformanceTracker::new(),
            instant: Instant::now(),
            time_factory: TimeFactory::new(),
            characters: Vec::new(),
            terrains: Vec::new()
        };

        let character = Character::new("stray_dog", CharacterKind::NPC(NPCType::Monster), &mut state).await;
        // character.translate(0.0, -0.5, 0.0, &mut state.scene);
        // dbg!(&character.objects);
        state.characters.push(character);
        let mut character = Character::new("stray_dog", CharacterKind::NPC(NPCType::Monster), &mut state).await;
        // dbg!(&character.objects);
        // panic!();
        character.translate(0.0, -0.5, 0.0, &mut state.scene);
        state.characters.push(character);


        if let Ok(terrain) = Terrain::load("c1", &state).await {
            state.terrains.push(terrain);
        }


        // panic!();
        state
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
                        // dbg!(self.performance_tracker.get_report());
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
        self.time_factory.tick();
        // self.performance_tracker.call_start("update");
        // self.performance_tracker.call_start("update_camera");
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.common_pipeline.uniforms.camera.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.common_pipeline.buffers.camera,
            0,
            bytemuck::cast_slice(&[self.common_pipeline.uniforms.camera]),
        );
        // self.performance_tracker.call_end("update_camera");
        // self.performance_tracker.call_start("update_light");
        let old_position: cgmath::Vector3<_> = self.common_pipeline.uniforms.light.position.into();
        self.common_pipeline.uniforms.light.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into();
        self.queue.write_buffer(&self.common_pipeline.buffers.light, 0, bytemuck::cast_slice(&[self.common_pipeline.uniforms.light]));
        // self.performance_tracker.call_end("update_light");

        // self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        // self.queue.write_buffer(
        //     &self.camera_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera_uniform]),
        // );
        // let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        // self.light_uniform.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into(); // UPDATED!
        // self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
        
        // self.performance_tracker.call_start("update_scene");
        self.scene.compute_world_matrices();
        self.scene.update_objects_buffers(&self.queue);
        // self.performance_tracker.call_end("update_scene");

        // self.performance_tracker.call_start("update_objects");

        let delta_ms = self.time_factory.get_delta();
        println!("delta {delta_ms}");

        for character in &self.characters {
            character.update(&mut self.scene);
        }

        for object in self.scene.get_all_objects() {
            if let Some(object_3d) = &mut object.object_3d {
                for instance in object_3d.get_instances() {
                    instance.update(delta_ms);
                }
                object_3d.update_skeleton(&self.queue);
            }
        }
        // self.performance_tracker.call_end("update_objects");
        // self.performance_tracker.call_end("update");
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // self.performance_tracker.call_start("render");
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

            // self.performance_tracker.call_start("render_draw_scene");
            render_pass.draw_scene(
                &self.queue,
                &mut self.scene, 
                &self.new_render_pipeline,
                &self.common_pipeline
            );
            // self.performance_tracker.call_end("render_draw_scene");

            render_pass.set_pipeline(&self.terrain_pipeline.pipeline);
            for terrain in &self.terrains {
                for chunk in terrain.get_terrain_meshes() {
                    render_pass.draw_custom_mesh(chunk, &self.common_pipeline);
                }
            }

            render_pass.set_pipeline(&self.water_pipeline.pipeline);
            for terrain in &self.terrains {
                for chunk in terrain.get_water_meshes() {
                    render_pass.draw_custom_mesh(chunk, &self.common_pipeline);
                }
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        // self.performance_tracker.call_start("render_present");
        output.present();
        // self.performance_tracker.call_end("render_present");
        // self.performance_tracker.call_end("render");
        Ok(())
    }
}