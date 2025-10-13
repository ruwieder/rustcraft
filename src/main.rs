mod core;
mod app;

use app::App;


fn main() {
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_local_timestamps()
        .with_level(log::LevelFilter::Info)
        .with_module_level("naga", log::LevelFilter::Warn)
        .with_module_level("rustcraft", log::LevelFilter::Trace)
        .with_module_level("wgpu", log::LevelFilter::Warn)
        // .with_module_level("calloop", log::LevelFilter::Info)
        .init().unwrap();
    log::info!("hello world bruh");
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    log::trace!("created init loop");
    let mut app = App::new();
    log::trace!("created app");
    event_loop.run_app(&mut app).expect("Event loop failed");
}