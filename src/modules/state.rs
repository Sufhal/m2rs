use std::{char, iter};
use cgmath::Rotation3;
use log::info;
use winit::keyboard::KeyCode;
use winit::{
    event::*,
    keyboard::PhysicalKey,
    window::Window,
};
use crate::modules::core::model::DrawCustomMesh;
use crate::modules::core::texture::{self, Texture};
use crate::modules::camera::camera;
use crate::modules::ui::ui::MetricData;
use crate::modules::utils::time_factory::TimeFragment;
use super::assets::gltf_loader::load_model_glb;
use super::camera::free_camera_controller::FreeCameraController;
use super::character::actor::Actor;
use super::character::character::{Character, CharacterKind, CharacterState, GetActor, NPCType, PCType, Sex};
use super::core::directional_light::{self, DirectionalLight};
use super::core::object_3d::{Object3D, TranslateWithScene};
use super::core::scene;
use super::pipelines::clouds_pipeline::CloudsPipeline;
use super::pipelines::common_pipeline::CommonPipeline;
use super::pipelines::shadow_pipeline::{self, ShadowPipeline};
use super::pipelines::simple_models_pipeline::SimpleModelPipeline;
use super::pipelines::skinned_models_pipeline::SkinnedModelPipeline;
use super::pipelines::sky_pipeline::SkyPipeline;
use super::pipelines::sun_pipeline::SunPipeline;
use super::pipelines::terrain_pipeline::TerrainPipeline;
use super::pipelines::water_pipeline::WaterPipeline;
use super::terrain::property::Properties;
use super::terrain::terrain::Terrain;
use super::ui::ui::UserInterface;
use super::utils::structs::KeyDebouncer;
use super::utils::time_factory::TimeFactory;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    sample_count: u32,
    #[allow(unused)]
    supported_sample_count: Vec<u32>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub common_pipeline: CommonPipeline,
    pub skinned_models_pipeline: SkinnedModelPipeline,
    pub simple_models_pipeline: SimpleModelPipeline,
    pub terrain_pipeline: TerrainPipeline,
    pub water_pipeline: WaterPipeline,
    pub sun_pipeline: SunPipeline,
    pub sky_pipeline: SkyPipeline,
    pub clouds_pipeline: CloudsPipeline,
    pub shadow_pipeline: ShadowPipeline,
    directional_light: DirectionalLight,
    camera: camera::Camera,
    projection: camera::Projection,
    pub camera_controller: FreeCameraController,
    pub mouse_pressed: bool,
    depth_texture: texture::Texture,
    pub multisampled_texture: wgpu::Texture,
    pub window: &'a Window,
    pub scene: scene::Scene,
    // performance_tracker: PerformanceTracker,
    // time: std::time::Instant,
    time_factory: TimeFactory,
    pub characters: Vec<Character>,
    pub actor: Option<Actor>,
    pub terrains: Vec<Terrain>,
    pub properties: Properties,
    pub ui: UserInterface,
    pub key_debouncer: KeyDebouncer,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> State<'a> {

        // let plane = Plane::new(1.0, 1.0, 2, 2);
        // dbg!(&plane, plane.indices.len(), plane.vertices.len());
        // panic!();


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
                    required_features: wgpu::Features::DEPTH_CLIP_CONTROL,
                    // required_features: wgpu::Features::all_webgpu_mask(),
                    // required_features: wgpu::Features::empty(),
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

        let surface_format_features = adapter.get_texture_format_features(surface_format);
        let supported_sample_count = surface_format_features.flags.supported_sample_counts();
        // let sample_count = 1;
        let sample_count = *supported_sample_count.iter().max().unwrap_or(&1);
        let sample_count = sample_count.min(4);

        let multisampled_texture = Self::create_multisampled_texture(
            &device, 
            sample_count, 
            (config.width, config.height), 
            config.format, 
            config.view_formats.clone()
        );

        let mut ui = UserInterface::new(&device, &config, &multisampled_texture, window.scale_factor() as f32);
        ui.std_out.push(format!("MSAA set to {sample_count}, supported values are {:?}", supported_sample_count));

        let directional_light = DirectionalLight::new(0.1, 100.0, [300.0, 400.0, 200.0], [376.0, 182.0, 641.0], &device);

        let camera = camera::Camera::new((376.0, 182.0, 641.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        // let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(28.0), 0.1, 100.0);
        let camera_controller = FreeCameraController::new(4.0, 0.4);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection, &directional_light);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, sample_count, "depth_texture");
        
        let common_pipeline = CommonPipeline::new(&device, &directional_light);
        let terrain_pipeline = TerrainPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let water_pipeline = WaterPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let sun_pipeline = SunPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let sky_pipeline = SkyPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let clouds_pipeline = CloudsPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let skinned_models_pipeline = SkinnedModelPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let simple_models_pipeline = SimpleModelPipeline::new(&device, &config, Some(texture::Texture::DEPTH_FORMAT), &multisampled_texture, &common_pipeline);
        let shadow_pipeline = ShadowPipeline::new(&device, &simple_models_pipeline);


        let mut scene = scene::Scene::new();

        // dbg!(&shaman_animations);

        // let model_objects = load_model_glb(
        //     "pack/pc/shaman_m/shaman_cheonryun.glb", 
        //     // "fox.glb", 
        //     &device, 
        //     &queue, 
        //     &skinned_models_pipeline,
        //     &simple_models_pipeline,
        // ).await.expect("unable to load");
        // for mut object in model_objects {
        //     if let Some(object3d) = &mut object.object3d {
        //         // let clips = load_animations(
        //         //     "shaman_wait_1.glb", 
        //         //     // "shaman_cheonryun.glb", 
        //         //     // "fox.glb", 
        //         //     &object_3d.model.skeleton
        //         // ).await.unwrap();
        //         // object_3d.set_animations(clips);
        //         // object_3d.model.animations = clips;
                
  
        //         // dbg!(&object.matrix);
        //         // println!("object {} have mesh", id);
        //         match object3d {
        //             Object3D::Skinned(skinned) => {
        //                 for i in 0..10 {
        //                     let instance = skinned.request_instance(&device);
        //                     instance.set_position(cgmath::Vector3::from([
        //                         0.5 + (i as f32 / 2.0),
        //                         0.0,
        //                         0.0
        //                     ]));
        //                     instance.take();
        //                 }
        //             },
        //             _ => ()
        //         };
        //     }
        //     scene.add(object);
        // }

        scene.compute_world_matrices();
        scene.update_objects_buffers(&queue);

        let properties = Properties::read().await.unwrap();
        
        let mut state = Self {
            surface,
            device,
            queue,
            config,
            size,
            common_pipeline,
            skinned_models_pipeline,
            simple_models_pipeline,
            terrain_pipeline,
            water_pipeline,
            sun_pipeline,
            sky_pipeline,
            clouds_pipeline,
            shadow_pipeline,
            directional_light,
            camera,
            projection,
            camera_controller,
            mouse_pressed: false, 
            depth_texture,
            multisampled_texture,
            sample_count,
            supported_sample_count,
            window,
            scene,
            // performance_tracker: PerformanceTracker::new(),
            time_factory: TimeFactory::new(),
            characters: Vec::new(),
            actor: None,
            terrains: Vec::new(),
            properties,
            ui,
            key_debouncer: KeyDebouncer::new(200.0)
        };

        // let mut character = Character::new("stray_dog", CharacterKind::NPC(NPCType::Monster), &mut state).await;
        // character.translate(384.0, 186.0, 640.0, &mut state.scene);
        // state.characters.push(character);

        let mut character = Character::new("shaman_cheonryun", CharacterKind::PC(PCType::Shaman(Sex::Male)), &mut state).await;
        character.translate(381.0, 200.0, 640.0, &mut state.scene);
        character.set_state(CharacterState::Wait, &mut state.scene);
        state.actor = Some(Actor::new(character.id.clone()));
        state.characters.push(character);




        // let q = Quaternion::from_angle_y(Deg(45.0));
        // character.rotate(q.s, q.v.x, q.v.y, q.v.z, &mut state.scene);
        // state.characters.push(character);
        // let mut character = Character::new("stray_dog", CharacterKind::NPC(NPCType::Monster), &mut state).await;
        // character.translate(385.0, 186.0, 640.0, &mut state.scene);
        // let q = Quaternion::from_angle_y(Deg(90.0));
        // character.rotate(q.s, q.v.x, q.v.y, q.v.z, &mut state.scene);
        // state.characters.push(character);
        // let mut character = Character::new("stray_dog", CharacterKind::NPC(NPCType::Monster), &mut state).await;
        // character.translate(384.0, 186.0, 641.0, &mut state.scene);
        // let q = Quaternion::from_angle_y(Deg(135.0));
        // character.rotate(q.s, q.v.x, q.v.y, q.v.z, &mut state.scene);
        // state.characters.push(character);
        // let mut character = Character::new("stray_dog", CharacterKind::NPC(NPCType::Monster), &mut state).await;
        // character.translate(385.0, 186.0, 641.0, &mut state.scene);
        // let q = Quaternion::from_angle_y(Deg(180.0));
        // character.rotate(q.s, q.v.x, q.v.y, q.v.z, &mut state.scene);
        // state.characters.push(character);
        


        let terrain = Terrain::load("c1", &mut state).await.unwrap();
        state.terrains.push(terrain);

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
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, self.sample_count, "depth_texture");
            self.projection.resize(new_size.width, new_size.height);
            self.ui.brush.resize_view(new_size.width as f32, new_size.height as f32, &self.queue);
            self.multisampled_texture = Self::create_multisampled_texture(
                &self.device, 
                self.sample_count, 
                (self.size.width, self.size.height),
                self.config.format,
                self.config.view_formats.clone()
            );
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
                    KeyCode::KeyI => {
                        if self.key_debouncer.hit(KeyCode::KeyI) {
                            self.ui.std_out.push("[I] pressed, environment toggle requested".to_string());
                            self.terrains.iter_mut().for_each(|terrain| {
                                terrain.environment.fog.toggle()
                            });
                        }
                    },
                    KeyCode::KeyC => {
                        if self.key_debouncer.hit(KeyCode::KeyC) {
                            self.ui.std_out.push("[C] pressed, camera control switch".to_string());
                            self.camera_controller.enabled = !self.camera_controller.enabled;
                        }
                    },
                    _ => {},
                };
                if self.camera_controller.enabled {
                    self.camera_controller.process_keyboard(*key, *state)
                } 
                else if let Some(actor) = &mut self.actor {
                    actor.process_keyboard(*key, *state)
                } else {
                    false
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if self.camera_controller.enabled {
                    self.camera_controller.process_scroll(delta);
                } else {
                    if let Some(actor) = &mut self.actor {
                        actor.orbit_controller.process_scroll(delta);
                    }
                }
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
        let update_call_fragment = TimeFragment::new();
        let elapsed_time = self.time_factory.elapsed_time_from_start();
        let delta_ms = self.time_factory.get_delta();
        
        if let Some(actor) = &mut self.actor {
            let character = self.characters.get_actor(actor);
            actor.apply_controls(character, &mut self.scene, &self.camera, delta_ms as f32);
            actor.orbit_controller.update_target(character.position);
        }
        for character in &mut self.characters {
            character.update(&mut self.scene, &self.terrains[0]);
        }

        if self.camera_controller.enabled {
            self.camera_controller.update_camera(&mut self.camera, dt);
        } else {
            if let Some(actor) = &mut self.actor {
                actor.orbit_controller.update_camera(&mut self.camera, &self.terrains[0]);
            }
        }
        self.common_pipeline.uniforms.camera.update_view_proj(&self.camera, &self.projection, &self.directional_light);
        self.queue.write_buffer(
            &self.common_pipeline.buffers.camera,
            0,
            bytemuck::cast_slice(&[self.common_pipeline.uniforms.camera]),
        );
        let old_position: cgmath::Vector3<_> = self.common_pipeline.uniforms.light.position.into();
        self.common_pipeline.uniforms.light.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into();
        self.queue.write_buffer(&self.common_pipeline.buffers.light, 0, bytemuck::cast_slice(&[self.common_pipeline.uniforms.light]));

        self.scene.compute_world_matrices();
        self.scene.update_objects_buffers(&self.queue);
        // let t = TimeFragment::new();
        // println!("t:{}", t.elapsed_ms());


        self.ui.update(delta_ms);
        

        for object in self.scene.get_all_objects_mut() {
            if let Some(object3d) = &mut object.object3d {
                match object3d {
                    Object3D::Skinned(skinned) => {
                        for instance in skinned.get_instances() {
                            instance.update(delta_ms);
                        }
                        skinned.update_skeleton(&self.queue);
                        skinned.update_instances(&self.queue);
                    },
                    Object3D::Simple(simple) => {
                        simple.update_instances(&self.queue);
                    }
                };
            }
        }

        for terrain in &mut self.terrains {
            terrain.update(elapsed_time, delta_ms as f32, &self.queue);
        }

        // self.shadow_pipeline.uniforms.directional_light = self.directional_light.uniform(self.projection.aspect);
        self.directional_light.update(self.common_pipeline.uniforms.camera.view_proj.into(), &self.terrains[0]);
        self.shadow_pipeline.uniforms.directional_light = self.directional_light.uniform_from_camera(self.common_pipeline.uniforms.camera.view_proj.into());
        self.queue.write_buffer(&self.shadow_pipeline.buffers.directional_light, 0, bytemuck::cast_slice(&[self.shadow_pipeline.uniforms.directional_light])); 
        self.queue.write_buffer(&self.common_pipeline.buffers.directional_light, 0, bytemuck::cast_slice(&[self.shadow_pipeline.uniforms.directional_light]));

        self.common_pipeline.uniforms.cycle.day_factor = self.terrains[0].environment.cycle.day_factor;
        self.common_pipeline.uniforms.cycle.night_factor = self.terrains[0].environment.cycle.night_factor;
        self.queue.write_buffer(&self.common_pipeline.buffers.cycle, 0, bytemuck::cast_slice(&[self.common_pipeline.uniforms.cycle]));
 
        self.common_pipeline.uniforms.sun = self.terrains[0].environment.sun.uniform;
        self.queue.write_buffer(&self.common_pipeline.buffers.sun, 0, bytemuck::cast_slice(&[self.common_pipeline.uniforms.sun]));

        self.common_pipeline.uniforms.fog = self.terrains[0].environment.fog.uniform();
        self.queue.write_buffer(&self.common_pipeline.buffers.fog, 0, bytemuck::cast_slice(&[self.common_pipeline.uniforms.fog]));

        self.ui.metrics.push_data(MetricData::UpdateCallTime(update_call_fragment.elapsed_ms()));
        self.ui.informations.position = [self.camera.position.x as i32, self.camera.position.y as i32, self.camera.position.z as i32];
        let current_cycle_time = self.terrains[0].environment.cycle.get_current_time();
        self.ui.informations.cycle_time = (current_cycle_time.0 as u32, current_cycle_time.1 as u32);

    }

    pub fn render(&mut self, mut fragment: TimeFragment) -> Result<(), wgpu::SurfaceError> {
        fragment.pause();
        // Avoiding tracking Queue::submit() called by Surface::get_current_texture(), the time execution is not revelant
        let output = self.surface.get_current_texture()?;
        fragment.resume();
        let render_call_fragment = TimeFragment::new();

        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        let multisampled_view = self.multisampled_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

        {
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.directional_light.cascade_textures[0].view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            shadow_pass.set_pipeline(&self.shadow_pipeline.pipeline);
            shadow_pass.set_bind_group(0, &self.shadow_pipeline.bind_group, &[]);

            for object in self.scene.get_all_objects() {
                if let Some(object3d) = &object.object3d {
                    match object3d {
                        Object3D::Simple(simple) => {
                            shadow_pass.set_vertex_buffer(1, simple.get_instance_buffer_slice());

                            for i in 0..simple.model.meshes.len() {
                                let mesh = &simple.model.meshes[i];
                                let mesh_bind_group = &simple.model.meshes_bind_groups[i];

                                shadow_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                shadow_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                shadow_pass.set_bind_group(1, &mesh_bind_group, &[]);
                                shadow_pass.set_bind_group(2, &simple.instances_bind_group, &[]);
                                shadow_pass.draw_indexed(0..mesh.num_elements, 0, 0..(simple.get_taken_instances_count() as u32));
                            }
                        },
                        _ => ()
                    
                    }
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &multisampled_view,
                    resolve_target: Some(&output_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
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

            render_pass.set_pipeline(&self.simple_models_pipeline.pipeline);
            render_pass.draw_scene_simple_models(
                &self.scene, 
                &self.common_pipeline
            );
            
            render_pass.set_pipeline(&self.skinned_models_pipeline.pipeline);
            render_pass.draw_scene_skinned_models(
                &self.scene, 
                &self.common_pipeline
            );

            render_pass.set_pipeline(&self.terrain_pipeline.pipeline);
            for terrain in &self.terrains {
                for chunk in terrain.get_terrain_meshes() {
                    render_pass.draw_custom_mesh(chunk, &self.common_pipeline);
                }
            }

            render_pass.set_pipeline(&self.water_pipeline.pipeline);
            for terrain in &self.terrains {
                for chunk in &terrain.chunks {
                    render_pass.set_vertex_buffer(1, chunk.depth_buffer.slice(..));
                    render_pass.draw_custom_mesh(&chunk.water_mesh, &self.common_pipeline);
                } 
            }

            render_pass.set_pipeline(&self.sky_pipeline.pipeline);
            for terrain in &self.terrains {
                render_pass.draw_custom_mesh(&terrain.environment.sky.mesh, &self.common_pipeline);
            }

            render_pass.set_pipeline(&self.clouds_pipeline.pipeline);
            for terrain in &self.terrains {
                render_pass.draw_custom_mesh(&terrain.environment.clouds.mesh, &self.common_pipeline);
            }

            render_pass.set_pipeline(&self.sun_pipeline.pipeline);
            for terrain in &self.terrains {
                render_pass.draw_custom_mesh(&terrain.environment.sun.mesh, &self.common_pipeline);
            }
        }

        self.ui.queue(&self.device, &self.queue);

        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &multisampled_view,
                        resolve_target: Some(&output_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            self.ui.brush.draw(&mut rpass);
        }

        self.ui.metrics.push_data(MetricData::RenderCallTime(render_call_fragment.elapsed_ms()));
        self.ui.metrics.push_data(MetricData::LogicalRenderTime(fragment.elapsed_ms()));
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn create_multisampled_texture(
        device: &wgpu::Device,
        sample_count: u32, 
        size: (u32, u32),
        format: wgpu::TextureFormat,
        view_formats: Vec<wgpu::TextureFormat>
    ) -> wgpu::Texture {
        let multisampled_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &view_formats,
            label: Some("Multisampled Texture"),
        });
        multisampled_texture
    }
}