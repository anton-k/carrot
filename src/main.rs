mod audio;
mod config;
mod ui;
use crate::audio::control::audio_control_channel;
use crate::audio::csound::Audio;
use crate::config::read_config_file;
use crate::ui::CarrotApp;
use crate::ui::parse::parse_config;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    match read_config_file() {
        Ok(file_content) => {
            let config = parse_config(&file_content.yaml).expect("Failed to parse config");
            let size = [config.config.size.width.0, config.config.size.height.0];

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size(size),
                ..Default::default()
            };
            let (ui_chan, csd_chan) = audio_control_channel();
            Audio::run(file_content.csd, csd_chan);

            eframe::run_native(
                "Carrot",
                options,
                Box::new(|_cc| Ok(Box::new(CarrotApp::new(&config, ui_chan)))),
            )
        }
        Err(msg) => {
            println!("Error: {:#?}", msg);
            panic!("Failed to parse congih")
        }
    }
}
