use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes};
use winit::dpi::PhysicalSize;
use std::time::Instant;
use futures::executor::block_on;

mod core;
use crate::core::render::renderer::Renderer;

use std::sync::Mutex;
use once_cell::sync::Lazy;

static WINDOW: Lazy<Mutex<Option<&'static Window>>> = Lazy::new(|| Mutex::new(None));

#[repr(C)]
#[derive(Copy, Clone)]
struct MovementState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

struct VoxelApp {
    renderer: Option<Renderer>,
    movement: MovementState,
    last_time: Instant,
    mouse_locked: bool,
    last_mouse_pos: (f32, f32),
}

impl VoxelApp {
    fn new() -> Self {
        Self {
            renderer: None,
            movement: MovementState {
                forward: false,
                backward: false,
                left: false,
                right: false,
                up: false,
                down: false,
            },
            last_time: Instant::now(),
            mouse_locked: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }

    // Helper method to get the window from the static
    fn get_window(&self) -> &'static Window {
        *WINDOW.lock().unwrap().as_ref().expect("Window not initialized")
    }
}

impl ApplicationHandler for VoxelApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Voxel Game in Rust + wgpu")
                .with_inner_size(PhysicalSize::new(1280, 720));
            
            // FIXME: absolute shit. leaking the window due to lifetime conflicts
            let window = event_loop.create_window(window_attributes).unwrap();
            let window_ref: &'static Window = Box::leak(Box::new(window));
            *WINDOW.lock().unwrap() = Some(window_ref);
            let renderer = block_on(Renderer::new(window_ref));
            
            self.renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let window = self.get_window();
        let renderer = self.renderer.as_mut().expect("Renderer should be initialized");
        
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    match key_code {
                        KeyCode::Escape => {
                            event_loop.exit();
                        }
                        KeyCode::KeyW => self.movement.forward = event.state == ElementState::Pressed,
                        KeyCode::KeyS => self.movement.backward = event.state == ElementState::Pressed,
                        KeyCode::KeyA => self.movement.left = event.state == ElementState::Pressed,
                        KeyCode::KeyD => self.movement.right = event.state == ElementState::Pressed,
                        KeyCode::Space => self.movement.up = event.state == ElementState::Pressed,
                        KeyCode::ShiftLeft => self.movement.down = event.state == ElementState::Pressed,
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    if state == ElementState::Pressed {
                        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                        window.set_cursor_visible(false);
                        self.mouse_locked = true;
                    } else {
                        let _ = window.set_cursor_grab(CursorGrabMode::None);
                        window.set_cursor_visible(true);
                        self.mouse_locked = false;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_locked {
                    let delta = (
                        position.x as f32 - self.last_mouse_pos.0,
                        position.y as f32 - self.last_mouse_pos.1,
                    );
                    renderer.update_camera(
                        0.016,
                        (
                            if self.movement.right { 1.0 } else { if self.movement.left { -1.0 } else { 0.0 } },
                            if self.movement.forward { 1.0 } else { if self.movement.backward { -1.0 } else { 0.0 } },
                            if self.movement.up { 1.0 } else { if self.movement.down { -1.0 } else { 0.0 } },
                        ),
                        delta,
                    );
                    self.last_mouse_pos = (position.x as f32, position.y as f32);
                }
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now.duration_since(self.last_time).as_secs_f64();
                self.last_time = now;

                renderer.update_camera(
                    delta_time,
                    (
                        if self.movement.right { 1.0 } else { if self.movement.left { -1.0 } else { 0.0 } },
                        if self.movement.forward { 1.0 } else { if self.movement.backward { -1.0 } else { 0.0 } },
                        if self.movement.up { 1.0 } else { if self.movement.down { -1.0 } else { 0.0 } },
                    ),
                    (0.0, 0.0),
                );

                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(window.inner_size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                    }
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.get_window();
        window.request_redraw();
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = VoxelApp::new();
    event_loop.run_app(&mut app).expect("Event loop failed");
}