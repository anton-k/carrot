mod audio;
mod config;
mod ui;
use crate::audio::control::audio_control_channel;
use crate::audio::csound::{Audio, ReadChannelMap};
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
            let app = CarrotApp::new(&config, ui_chan);
            Audio::run(
                file_content.csd,
                csd_chan,
                ReadChannelMap::new(&config.csound.read),
                app.channels.get_all_channels().clone(),
            );

            eframe::run_native("Carrot", options, Box::new(|_cc| Ok(Box::new(app))))
        }
        Err(msg) => {
            println!("Error: {:#?}", msg);
            panic!("Failed to parse congih")
        }
    }
}
