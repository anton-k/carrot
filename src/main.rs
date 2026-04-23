mod ui;
use std::env;
use std::fs;
/*
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
*/

use eframe::egui;
use egui_knob::{Knob, KnobStyle, LabelPosition};

struct KnobApp {
    value: f32,
}

impl Default for KnobApp {
    fn default() -> Self {
        Self { value: 0.5 }
    }
}

impl eframe::App for KnobApp {
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let knob = Knob::new(&mut self.value, 0.0, 1.0, KnobStyle::Wiper)
                .with_size(50.0)
                .with_font_size(14.0)
                .with_colors(
                    egui::Color32::GRAY,
                    egui::Color32::WHITE,
                    egui::Color32::WHITE,
                )
                .with_stroke_width(3.0)
                .with_label("Volume", LabelPosition::Top);

            ui.add(knob);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Minimal",
        options,
        Box::new(|_cc| Ok(Box::new(KnobApp::default()) as Box<dyn eframe::App>)),
    )
    .unwrap();
}

/*
use eframe::egui; // Import necessary parts of eframe and egui

// The main function where our program starts
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "egui Demo",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

// This struct holds the data (state) for our application.
#[derive(Default)]
struct MyApp {
    label: String,
    value: f32,
    // Add a boolean field for the checkbox state
    show_extra_info: bool,
    // Add a field to store the selected color choice.
    selected_color: ColorChoice,
    // Let's add another state variable for demonstration
    counter: i32,
    // Add the current mode field
    current_mode: AppMode,
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
enum ColorChoice {
    Red,
    Green,
    #[default]
    Blue,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
enum AppMode {
    #[default]
    View,
    Edit,
    Settings,
}

// We implement the `eframe::App` trait for our struct.
impl eframe::App for MyApp {
    // The `update` function is called repeatedly, once per frame.
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // --- Top Panel for Mode Switching (Example) ---
        egui::TopBottomPanel::top("mode_switcher").show_inside(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Mode:");
                // Use radio buttons to switch the app mode state (we'll use self.current_mode)
                ui.radio_value(&mut self.current_mode, AppMode::View, "View");
                ui.radio_value(&mut self.current_mode, AppMode::Edit, "Edit");
                ui.radio_value(&mut self.current_mode, AppMode::Settings, "Settings");
            });
        });

        // --- Central Panel for Content based on Mode ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Current Mode: {:?}", self.current_mode)); // Show the mode
            ui.separator();

            // `match` allows us to render different UI based on the current mode
            match self.current_mode {
                AppMode::View => {
                    ui.label("Viewing Data (Read Only):");
                    ui.label(format!("Label: {}", self.label));
                    ui.label(format!("Value: {:.1}", self.value));
                    // Show extra info conditionally based on checkbox state (from Settings)
                    if self.show_extra_info {
                        ui.label(format!("Counter: {}", self.counter));
                        ui.label(format!("Color: {:?}", self.selected_color));
                    } else {
                        ui.label("(Enable 'Show Advanced Info' in Settings to see more)");
                    }
                    if ui.button("Switch to Edit Mode").clicked() {
                        self.current_mode = AppMode::Edit;
                    }
                }

                AppMode::Edit => {
                    ui.label("Editing Data:");
                    // Example: Using a Grid for alignment in Edit mode (see Layout section)
                    egui::Grid::new("edit_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Label:"); // Col 1
                            ui.text_edit_singleline(&mut self.label); // Col 2
                            ui.end_row();

                            ui.label("Value:"); // Col 1
                            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0)); // Col 2
                            ui.end_row();

                            ui.label("Counter:"); // Col 1
                            ui.horizontal(|ui| {
                                // Use horizontal layout within grid cell
                                if ui.button("+").clicked() {
                                    self.counter += 1;
                                }
                                ui.label(format!("{}", self.counter));
                                if ui.button("-").clicked() {
                                    self.counter -= 1;
                                }
                            }); // Col 2
                            ui.end_row();
                        });
                }

                AppMode::Settings => {
                    ui.label("Application Settings:");
                    ui.separator();

                    // --- Checkbox Example ---
                    // `ui.checkbox(&mut self.show_extra_info, "Show Advanced Info on View Tab");`
                    //   `&mut self.show_extra_info`: Mutably borrows the boolean field. Clicking toggles this value.
                    //   `"..."`: The label displayed next to the checkbox.
                    ui.checkbox(&mut self.show_extra_info, "Show Advanced Info on View Tab");

                    ui.separator();

                    // --- RadioButton Example ---
                    ui.label("Color Scheme:");
                    ui.horizontal(|ui| {
                        // `ui.radio_value(&mut self.selected_color, ColorChoice::Red, "Red");`
                        //   `&mut self.selected_color`: Mutably borrows the enum field holding the current choice.
                        //   `ColorChoice::Red`: The specific value this radio button represents.
                        //   `"Red"`: The label for this button.
                        // Clicking this updates `self.selected_color` *only if* it's not already `Red`.
                        ui.radio_value(&mut self.selected_color, ColorChoice::Red, "Red");
                        ui.radio_value(&mut self.selected_color, ColorChoice::Green, "Green");
                        ui.radio_value(&mut self.selected_color, ColorChoice::Blue, "Blue");
                    });
                    ui.label(format!("Selected color: {:?}", self.selected_color)); // Display choice

                    ui.separator();

                    // --- Reset Button Example ---
                    if ui.button("Reset All State").clicked() {
                        // Replace `self`'s contents with a brand new default instance.
                        // Requires `*` to dereference `self` for assignment.
                        *self = MyApp::default();
                        // Keep the mode as Settings after reset
                        self.current_mode = AppMode::Settings;
                    }
                }
            } // End of match expression
        });
    }
}
*/
