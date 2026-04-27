mod ui;
use crate::ui::CarrotApp;
use crate::ui::parse::parse_config;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::env;
use std::fs;

fn main() -> eframe::Result<()> {
    match read_config_file() {
        Ok(file_content) => {
            let config = parse_config(&file_content.yaml).expect("Failed to parse config");
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

pub struct ConfigFileContent {
    pub yaml: String,
    pub csd: Option<String>,
}

fn read_config_file() -> Result<ConfigFileContent, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        Err("Error: Should have at least one argument for config file".to_string())
    } else {
        let file_path = args[1].clone();
        if is_yaml(&file_path) {
            read_from_yaml(&file_path)
        } else if is_csd(&file_path) {
            read_from_csd(&file_path)
        } else {
            Err("Wrong file format. Should be .yaml or .csd".to_string())
        }
    }
}

fn is_yaml(file: &str) -> bool {
    file.ends_with(".yaml")
}

fn is_csd(file: &str) -> bool {
    file.ends_with(".csd")
}

fn read_from_yaml(file_path: &str) -> Result<ConfigFileContent, String> {
    let yaml = fs::read_to_string(file_path).map_err(|err| err.to_string())?;
    Ok(ConfigFileContent { yaml, csd: None })
}

fn read_from_csd(file_path: &String) -> Result<ConfigFileContent, String> {
    let csd = fs::read_to_string(file_path).map_err(|err| err.to_string())?;
    let yaml = read_xml_tag_content("Carrot", &csd)?;
    Ok(ConfigFileContent {
        yaml,
        csd: Some(csd),
    })
}

fn read_xml_tag_content(tag: &str, xml: &str) -> Result<String, String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut current_tag = String::new();
    let tag_bytes = tag.as_bytes();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == tag_bytes {
                    // Reads until </name>, automatically decoding the text
                    let text = reader.read_text(e.name()).expect("Failed to read text");
                    return Ok(text.to_string());
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break, // End of file
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            _ => (), // Ignore other events like Comments or CDATA
        }
        buf.clear(); // Reuse buffer for performance
    }
    Err("No tag <Carrot> for UI in the csd file".to_string())
}
