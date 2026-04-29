mod config;
mod ui;
use crate::config::read_config_file;
use crate::ui::CarrotApp;
use crate::ui::parse::parse_config;

use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
    match read_config_file() {
        Ok(file_content) => {
            let config = parse_config(&file_content.yaml).expect("Failed to parse config");
            let size = [config.config.size.width.0, config.config.size.height.0];

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size(size),
                ..Default::default()
            };

            let csd = Arc::new(Mutex::new(file_content.csd));
            let csd_perf = Arc::clone(&csd);
            std::thread::spawn(move || while !csd_perf.lock().unwrap().perform_ksmps() {});

            let csd_ui = Arc::clone(&csd);

            eframe::run_native(
                "Carrot",
                options,
                Box::new(|_cc| Ok(Box::new(CarrotApp::new(&config, csd_ui)))),
            )
        }
        Err(msg) => {
            println!("Error: {:#?}", msg);
            panic!("Failed to parse congih")
        }
    }
}
