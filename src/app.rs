use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes};
use winit::dpi::PhysicalSize;
use std::time::Instant;
use futures::executor::block_on;

use crate::core::render::renderer::Renderer;
use crate::WINDOW_PTR;

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

pub struct App {
    renderer: Option<Renderer>,
    movement: MovementState,
    last_time: Instant,
    mouse_locked: bool,
    last_mouse_pos: (f32, f32),
}

impl App {
    pub fn new() -> Self {
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
        *WINDOW_PTR.lock().unwrap().as_ref().expect("Window not initialized")
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Rustcraft")
                .with_inner_size(PhysicalSize::new(1280, 720));
            
            // FIXME: absolute shit. leaking the window due to lifetime conflicts
            let window = event_loop.create_window(window_attributes).unwrap();
            let window_ref: &'static Window = Box::leak(Box::new(window));
            *WINDOW_PTR.lock().unwrap() = Some(window_ref);
            let renderer = block_on(Renderer::new(window_ref));
            
            self.renderer = Some(renderer);
        }
    }
    
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(renderer) = self.renderer.take() {
            drop(renderer);
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
                    println!("pos: {:?}", renderer.camera.pos);
                    println!("rot: {:?}", renderer.camera.rot);
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