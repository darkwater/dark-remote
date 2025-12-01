#![warn(clippy::all, rust_2018_idioms)]

mod connection;
mod utils;

mod app;
pub use app::DarkRemoteApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: egui_winit::winit::platform::android::activity::AndroidApp) {
    use eframe::{NativeOptions, Renderer};

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = NativeOptions {
        android_app: Some(app),
        renderer: Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native("dark remote", options, Box::new(|cc| Ok(Box::new(DarkRemoteApp::new(cc)))))
        .expect("Failed to start eframe");
}
