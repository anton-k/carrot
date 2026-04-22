mod ui;
use std::env;
use std::fs;

use crate::ui::parse::parse_config;

fn read_config_file() -> Result<String, String> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        Err("Error: Should have at least one argument for config file".to_string())
    } else {
        let file_path = args[1].clone();
        fs::read_to_string(file_path).map_err(|err| err.to_string())
    }
}

fn main() {
    match read_config_file() {
        Ok(file) => println!("{:#?}", parse_config(&file)),
        Err(msg) => println!("{:#?}", msg),
    }
}
