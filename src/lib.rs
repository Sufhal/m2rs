pub mod modules;
use modules::state::State;
use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    keyboard::{PhysicalKey, KeyCode},
    window::WindowBuilder,
};
use log::info;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll); // <- solves the issue of performance drop when refocusing the window in wasm export
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("M2RS");

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("render")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;
    let mut last_render_time = instant::Instant::now();  // NEW!
    let mut surface_configured = false;

    event_loop
        .run(move |event, control_flow| {
            control_flow.set_control_flow(ControlFlow::Poll);
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion{ delta, },
                    .. // We're not using device_id currently
                } => if state.mouse_pressed {
                    state.camera_controller.process_mouse(delta.0, delta.1)
                },
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() && !state.input(event)  => {
                    if !state.input(event) {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                surface_configured = true;
                                state.resize(*physical_size);
                            },
                            WindowEvent::Focused(focused) => {
                                info!("window is focused : {focused}");
                                // control_flow.set_control_flow(ControlFlow::Wait);
                            },
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                state.window().request_redraw();

                                #[cfg(target_arch = "wasm32")]
                                {
                                    use winit::dpi::LogicalSize;

                                    web_sys::window()
                                        .and_then(|win| {
                                            state.window.request_inner_size(LogicalSize::new(
                                                win.inner_width().ok()?.as_f64()?,
                                                win.inner_height().ok()?.as_f64()?,
                                            ));
                                            Some(())
                                        })
                                        .expect("Couldn't resize window");
                                }

                                if !surface_configured {
                                    return;
                                }

                                let now = instant::Instant::now();
                                let dt = now - last_render_time;
                                last_render_time = now;
                                state.update(dt);
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
