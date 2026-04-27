mod ui;
use crate::ui::CarrotApp;
use crate::ui::parse::parse_config;
use std::env;
use std::fs;

fn main() -> eframe::Result<()> {
    match read_config_file() {
        Ok(file) => {
            let config = parse_config(&file).expect("Failed to parse config");
            let size = [config.config.size.width.0, config.config.size.height.0];

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size(size),
                ..Default::default()
            };

            eframe::run_native(
                "Carrot",
                options,
                Box::new(|_cc| Ok(Box::new(CarrotApp::new(&config)))),
            )
        }
        Err(msg) => {
            println!("Error: {:#?}", msg);
            panic!("Failed to parse congih")
        }
    }
}

fn read_config_file() -> Result<String, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        Err("Error: Should have at least one argument for config file".to_string())
    } else {
        let file_path = args[1].clone();
        fs::read_to_string(file_path).map_err(|err| err.to_string())
    }
}
