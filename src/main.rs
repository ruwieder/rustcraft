mod core;
mod app;
mod world;
use app::App;

// gives about +5% performance in comparison to default, keep for now
use tcmalloc_better::TCMalloc;
#[global_allocator]
static GLOBAL: TCMalloc = TCMalloc;

fn main() {
    TCMalloc::process_background_actions_thread();
    
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_local_timestamps()
        .with_level(log::LevelFilter::Warn)
        // .with_module_level("rustcraft", log::LevelFilter::Trace)
        .init().unwrap();
    
    log::info!("hello world bruh");
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    log::trace!("created init loop");
    let mut app = App::new();
    log::trace!("created app");
    event_loop.run_app(&mut app).expect("Event loop failed");
}