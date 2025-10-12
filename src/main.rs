mod core;
mod app;

use app::App;


fn main() {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop failed");
}