use client::app::App;
use eframe::{run_native, NativeOptions};

fn main() {
    let native_options = NativeOptions::default();
    run_native(
        "chat",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
    .unwrap();
}
