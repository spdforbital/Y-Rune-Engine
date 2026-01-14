#![allow(unsafe_op_in_unsafe_fn)]

use ash::vk;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod engine;
mod vulkan;
use engine::{Engine, InputState};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let fullscreen = true;
    let mut crosshair_enabled = true;
    let msaa_samples = vk::SampleCountFlags::TYPE_4;  
    let mut engine = Engine::new(&event_loop, fullscreen, crosshair_enabled, msaa_samples);
    let mut input = InputState::default();
    let mut debug_mode = true;
    engine.set_debug(debug_mode);
    let mut sun_enabled = true;  
    engine.set_sun_enabled(sun_enabled);
    let mut bloom_enabled = true;
    engine.set_bloom_enabled(bloom_enabled);
    
    let skip_main_menu = false;
    if skip_main_menu {
        engine.load_scene("scene2");
    }

    let mut last_cursor_pos: Option<(f64, f64)> = None;
    let cap_fps = true;
    let target_frame = std::time::Duration::from_micros(1_000_000 / 60);
    let mut last_frame = std::time::Instant::now();

     
    engine.window().set_cursor_visible(false);
    match engine.window().set_cursor_grab(winit::window::CursorGrabMode::Confined) {
        Ok(_) => {},
        Err(_) => {
              
             let _ = engine.window().set_cursor_grab(winit::window::CursorGrabMode::None);
        }
    }

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    target.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    engine.handle_resize();
                }
                Event::DeviceEvent {
                    event: winit::event::DeviceEvent::MouseMotion { delta, .. },
                    ..
                } => {
                      
                     if !engine.is_interface_open() {
                         engine.handle_mouse_delta(delta.0, delta.1);
                     }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    use winit::event::{ElementState, MouseButton};
                    if button == MouseButton::Left {
                         input.mouse_down = state.is_pressed();
                         match state {
                             ElementState::Pressed => {
                                 if engine.is_interface_open() {
                                     if let Some((x, y)) = last_cursor_pos {
                                         engine.handle_mouse_down(x, y);
                                     }
                                 } else {
                                     let size = engine.window().inner_size();
                                     engine.handle_mouse_down(size.width as f64 / 2.0, size.height as f64 / 2.0);
                                 }
                             }
                             ElementState::Released => {
                                 if engine.is_interface_open() {
                                     if let Some((x, y)) = last_cursor_pos {
                                         engine.handle_mouse_up(x, y);
                                     }
                                 } else {
                                     let size = engine.window().inner_size();
                                     engine.handle_mouse_up(size.width as f64 / 2.0, size.height as f64 / 2.0);
                                 }
                             }
                         }
                    } else if button == MouseButton::Right {
                        if state == ElementState::Pressed {
                            engine.handle_right_click();
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                     
                    last_cursor_pos = Some((position.x, position.y));
                    input.cursor_pos = [position.x as f32, position.y as f32];
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => {
                    if let PhysicalKey::Code(code) = event.physical_key {
                        let pressed = event.state.is_pressed();
                        match code {
                            KeyCode::Escape => {
                                if pressed {
                                     
                                     
                                     
                                    target.exit(); 
                                }
                            }
                            KeyCode::KeyW => input.forward = pressed,
                            KeyCode::KeyS => input.backward = pressed,
                            KeyCode::KeyA => input.left = pressed,
                            KeyCode::KeyD => input.right = pressed,
                            KeyCode::Space => input.jump = pressed,
                            KeyCode::ShiftLeft | KeyCode::ShiftRight => input.sprint = pressed,
                            KeyCode::KeyL => {
                                if pressed {
                                    sun_enabled = !sun_enabled;
                                    engine.set_sun_enabled(sun_enabled);
                                }
                            }
                            KeyCode::KeyF => {
                                if pressed {
                                    engine.toggle_inventory();
                                     
                                }
                            }
                            KeyCode::KeyC => {
                                if pressed {
                                    crosshair_enabled = !crosshair_enabled;
                                    engine.set_crosshair_enabled(crosshair_enabled);
                                }
                            }
                            KeyCode::KeyB => {
                                if pressed {
                                    bloom_enabled = !bloom_enabled;
                                    engine.set_bloom_enabled(bloom_enabled);
                                    println!("Bloom: {}", bloom_enabled);
                                }
                            }
                            KeyCode::KeyE => input.interact = pressed,
                            KeyCode::KeyG => {
                                input.drop = pressed;
                            }
                            KeyCode::KeyV => {
                                if pressed {
                                    engine.toggle_noclip();
                                }
                            }
                            KeyCode::F3 => {
                                if pressed {
                                    debug_mode = !debug_mode;
                                    engine.set_debug(debug_mode);
                                }
                            }
                            KeyCode::Digit1 => { if pressed { engine.select_hotbar_slot(0); } }
                            KeyCode::Digit2 => { if pressed { engine.select_hotbar_slot(1); } }
                            KeyCode::Digit3 => { if pressed { engine.select_hotbar_slot(2); } }
                            KeyCode::Digit4 => { if pressed { engine.select_hotbar_slot(3); } }
                            KeyCode::Digit5 => { if pressed { engine.select_hotbar_slot(4); } }
                            _ => {}
                        }
                    }
                }
                Event::AboutToWait => {
                    let interface_open = engine.is_interface_open();
                    engine.window().set_cursor_visible(interface_open);
                    let grab_mode = if interface_open {
                        winit::window::CursorGrabMode::None
                    } else {
                        winit::window::CursorGrabMode::Confined
                    };
                    if let Err(_) = engine.window().set_cursor_grab(grab_mode) {
                        // Fallback if confined is not supported
                        if !interface_open {
                             let _ = engine.window().set_cursor_grab(winit::window::CursorGrabMode::Locked);
                        }
                    }

                    if cap_fps {
                        let elapsed = last_frame.elapsed();
                        if elapsed < target_frame {
                            std::thread::sleep(target_frame - elapsed);
                        }
                        last_frame = std::time::Instant::now();
                    }
                    engine.window().request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => unsafe {
                    engine.render(&input);
                },
                _ => (),
            }
        })
        .unwrap();
}
