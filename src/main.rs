use winit::{
    event::{
        Event, WindowEvent, WindowEvent::{KeyboardInput}, ElementState, VirtualKeyCode, MouseButton,
        MouseScrollDelta,
    },
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, WindowBuilder},
};
use std::time::{Instant, Duration};
use futures::executor::block_on;

mod core;
use core::*;

use renderer::{Renderer, Camera};
use world::World;

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

fn main() {
    // env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Voxel Game in Rust + wgpu")
        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let mut renderer = block_on(Renderer::new(&window));
    let mut movement = MovementState {
        forward: false,
        backward: false,
        left: false,
        right: false,
        up: false,
        down: false,
    };
    let mut last_time = Instant::now();
    let mut mouse_locked = false;
    let mut last_mouse_pos = (0.0, 0.0);

    event_loop.run_app((), move |event, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                },
                ..
            } => {
                window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
                window.set_cursor_visible(false);
                mouse_locked = true;
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button: MouseButton::Left,
                    ..
                },
                ..
            } => {
                window.set_cursor_grab(CursorGrabMode::None).unwrap();
                window.set_cursor_visible(true);
                mouse_locked = false;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                if mouse_locked {
                    let delta = (
                        position.x as f32 - last_mouse_pos.0,
                        position.y as f32 - last_mouse_pos.1,
                    );
                    renderer.update_camera(
                        0.016, // approx 60 FPS
                        (
                            if movement.right { 1.0 } else { if movement.left { -1.0 } else { 0.0 } },
                            if movement.forward { 1.0 } else { if movement.backward { -1.0 } else { 0.0 } },
                            if movement.up { 1.0 } else { if movement.down { -1.0 } else { 0.0 } },
                        ),
                        delta,
                    );
                    last_mouse_pos = (position.x as f32, position.y as f32);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                    ..
                },
                ..
            } => {
                match key {
                    VirtualKeyCode::W => movement.forward = state == ElementState::Pressed,
                    VirtualKeyCode::S => movement.backward = state == ElementState::Pressed,
                    VirtualKeyCode::A => movement.left = state == ElementState::Pressed,
                    VirtualKeyCode::D => movement.right = state == ElementState::Pressed,
                    VirtualKeyCode::Space => movement.up = state == ElementState::Pressed,
                    VirtualKeyCode::Shift => movement.down = state == ElementState::Pressed,
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_time).as_secs_f64();
                last_time = now;

                renderer.update_camera(
                    delta_time,
                    (
                        if movement.right { 1.0 } else { if movement.left { -1.0 } else { 0.0 } },
                        if movement.forward { 1.0 } else { if movement.backward { -1.0 } else { 0.0 } },
                        if movement.up { 1.0 } else { if movement.down { -1.0 } else { 0.0 } },
                    ),
                    (0.0, 0.0),
                );

                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(window.inner_size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}