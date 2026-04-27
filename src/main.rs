mod ui;
use crate::ui::parse::parse_config;
use crate::ui::types::{
    Channel, ChannelMap, PrimUi, Rect, UiConfig, WidgetRect, get_ui_rect, get_ui_state,
};
use eframe::egui;
use egui_knob::{Knob, KnobStyle};
use std::collections::HashMap;
use std::env;
use std::fs;

struct CarrotApp {
    pub channels: ChannelMap,
    pub channels_to_update: Vec<ChannelUpdate>,
    pub prims: Vec<WidgetRect<PrimUi>>,
}

impl CarrotApp {
    pub fn apply_updates(&mut self) {
        let mut post_updates = Vec::new();
        while !self.channels_to_update.is_empty() {
            if let Some(chan_update) = self.channels_to_update.pop() {
                apply_update(&chan_update, &mut post_updates);
            }
        }
        self.channels_to_update = post_updates;
    }
}

fn apply_update(chan_update: &ChannelUpdate, post_updates: &mut Vec<ChannelUpdate>) {
    match chan_update.value {
        UpdateValue::Float { value, post_update } => {
            println!("Update float: {:?}: {:?}", chan_update.channel, value);
            post_update.iter().for_each(|post_val| {
                post_updates.push(ChannelUpdate::update_float(&chan_update.channel, *post_val))
            });
        }
        UpdateValue::Bool { value, post_update } => {
            println!("Update bool: {:?}: {:?}", chan_update.channel, value);
            post_update.iter().for_each(|post_val| {
                post_updates.push(ChannelUpdate::update_bool(&chan_update.channel, *post_val))
            });
        }
    }
}

struct ChannelUpdate {
    pub channel: Channel,
    pub value: UpdateValue,
}

impl ChannelUpdate {
    pub fn update_float(channel: &Channel, value: f32) -> ChannelUpdate {
        ChannelUpdate::update_float_with_post(channel, value, None)
    }

    pub fn update_float_with_post(
        channel: &Channel,
        value: f32,
        post_update: Option<f32>,
    ) -> ChannelUpdate {
        ChannelUpdate {
            channel: channel.clone(),
            value: UpdateValue::Float { value, post_update },
        }
    }

    pub fn update_bool(channel: &Channel, value: bool) -> ChannelUpdate {
        ChannelUpdate::update_bool_with_post(channel, value, None)
    }

    pub fn update_bool_with_post(
        channel: &Channel,
        value: bool,
        post_update: Option<bool>,
    ) -> ChannelUpdate {
        ChannelUpdate {
            channel: channel.clone(),
            value: UpdateValue::Bool { value, post_update },
        }
    }
}

enum UpdateValue {
    Float {
        value: f32,
        post_update: Option<f32>,
    },
    Bool {
        value: bool,
        post_update: Option<bool>,
    },
}

impl CarrotApp {
    fn new(config: &UiConfig) -> Self {
        let ui_with_rect = get_ui_rect(&Rect::unit(), &config.ui);
        println!("{:#?}", &ui_with_rect);
        let ui_state = get_ui_state(&ui_with_rect);
        Self {
            channels: ui_state.channels,
            channels_to_update: Vec::new(),
            prims: ui_state.prims,
        }
    }
}

fn scale_by_size(rect: &mut egui::Rect, size: &egui::Vec2) {
    rect.min = scale_pos(&rect.min, size);
    rect.max = scale_pos(&rect.max, size);
}

fn scale_pos(pos: &egui::Pos2, size: &egui::Vec2) -> egui::Pos2 {
    egui::Pos2 {
        x: pos.x * size.x,
        y: pos.y * size.y,
    }
}

impl eframe::App for CarrotApp {
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            let channels = &mut self.channels;
            let channels_to_update = &mut self.channels_to_update;
            // report a bug: why available size is 16 units smaller?
            let size = ui.available_size() + egui::Vec2::new(16.0, 16.0);
            self.prims.iter().for_each(|prim| {
                add_widget(
                    channels,
                    channels_to_update,
                    ui,
                    &size,
                    &prim.rect,
                    &prim.item,
                );
            });
            self.apply_updates();
        });
    }
}

fn add_widget(
    channels: &mut ChannelMap,
    channels_to_update: &mut Vec<ChannelUpdate>,
    ui: &mut egui::Ui,
    size: &egui::Vec2,
    widget_rect: &Rect,
    prim: &PrimUi,
) {
    let mut rect = egui::Rect::from(*widget_rect);
    scale_by_size(&mut rect, size);

    match prim {
        PrimUi::Space => {}

        PrimUi::Button { channel, text } => add_button(
            channels_to_update,
            ui,
            &rect,
            channel,
            &get_button_name(channel, text),
        ),
        PrimUi::Knob { channel } => {
            add_knob(channels, channels_to_update, ui, &rect, channel, &channel.0)
        }
        PrimUi::Slider { channel } => {
            add_slider(channels, channels_to_update, ui, &rect, channel, &channel.0)
        }
        PrimUi::Toggle { channel, text } => add_toggle(
            channels,
            channels_to_update,
            ui,
            &rect,
            channel,
            &get_button_name(channel, text),
        ),
        PrimUi::Select {
            channel: _,
            text: _,
        } => todo!(),
        PrimUi::Label { text, size: _ } => add_label(ui, &rect, text),
        PrimUi::Image { file: _ } => todo!(), // add_image(ui, &rect, file),
    }
}

fn get_button_name(channel: &Channel, text: &str) -> String {
    if text.is_empty() {
        channel.0.clone()
    } else {
        text.to_string()
    }
}

fn add_button(
    channels_to_update: &mut Vec<ChannelUpdate>,
    ui: &mut egui::Ui,
    rect: &egui::Rect,
    channel: &Channel,
    text: &str,
) {
    let response = ui.put(*rect, egui::Button::new(text).sense(egui::Sense::drag()));
    if response.drag_started() {
        update_bool(channels_to_update, channel, true);
    }
    if response.drag_stopped() {
        update_bool(channels_to_update, channel, false);
    }
}

fn add_toggle(
    channels: &mut ChannelMap,
    channels_to_update: &mut Vec<ChannelUpdate>,
    ui: &mut egui::Ui,
    rect: &egui::Rect,
    channel: &Channel,
    text: &str,
) {
    let chan_ref = channels.get_mut_bool(channel);
    let response = ui.put(*rect, egui::Checkbox::new(chan_ref, text));
    if response.clicked() {
        update_bool(channels_to_update, channel, *chan_ref);
    }
}

fn add_label(ui: &mut egui::Ui, rect: &egui::Rect, text: &str) {
    ui.put(*rect, egui::Label::new(text));
}
/*
fn add_image(ui: &mut egui::Ui, rect: &egui::Rect, file: &str) {
    ui.put(*rect, egui::Image::new(egui::include_image!(file)));
}
*/
fn add_knob(
    channels: &mut ChannelMap,
    channels_to_update: &mut Vec<ChannelUpdate>,
    ui: &mut egui::Ui,
    rect: &egui::Rect,
    channel: &Channel,
    _text: &str,
) {
    let size = rect.size();
    let chan_ref = channels.get_mut_float(channel);
    let knob = Knob::new(
        chan_ref,
        0.0,
        1.0,
        KnobStyle::Wiper,
    )
    .with_size(0.8 * size.x.min(size.y))
    .with_font_size(14.0)
    .with_sweep_range(0.1, 0.8)
    .with_colors(
        egui::Color32::GRAY,
        egui::Color32::WHITE,
        egui::Color32::WHITE,
    )
    .with_stroke_width(3.0)
    // .with_label(text, LabelPosition::Top)
    ;

    let response = ui.put(*rect, knob);
    if response.clicked() {
        println!("Clicked!");
    };
    check_update_float(&response, channels_to_update, channel, *chan_ref);
}

fn add_slider(
    channels: &mut ChannelMap,
    channels_to_update: &mut Vec<ChannelUpdate>,
    ui: &mut egui::Ui,
    rect: &egui::Rect,
    channel: &Channel,
    _text: &str,
) {
    let size = rect.size();
    let chan_ref = channels.get_mut_float(channel);
    let slider = if size.x > size.y {
        ui.spacing_mut().slider_width = 0.9 * size.x;
        egui::Slider::new(chan_ref, 0.0..=1.0).show_value(false)
    } else {
        ui.spacing_mut().slider_width = 0.9 * size.y;
        egui::Slider::new(chan_ref, 0.0..=1.0)
            .vertical()
            .show_value(false)
    };
    let mut rect_pad = *rect;
    rect_pad.min.x += size.x * 0.05;
    rect_pad.min.y += size.y * 0.05;
    let response = ui.put(rect_pad, slider);
    check_update_float(&response, channels_to_update, channel, *chan_ref);
}

fn check_update_float(
    response: &egui::Response,
    channels_to_update: &mut Vec<ChannelUpdate>,
    channel: &Channel,
    value: f32,
) {
    check_update_float_with_post_update(response, channels_to_update, channel, value, None);
}

fn check_update_float_with_post_update(
    response: &egui::Response,
    channels_to_update: &mut Vec<ChannelUpdate>,
    channel: &Channel,
    value: f32,
    post_update: Option<f32>,
) {
    if response.changed() {
        channels_to_update.push(ChannelUpdate::update_float_with_post(
            channel,
            value,
            post_update,
        ));
    }
}

fn check_update_bool(
    response: &egui::Response,
    channels_to_update: &mut Vec<ChannelUpdate>,
    channel: &Channel,
    value: bool,
) {
    check_update_bool_with_post_update(response, channels_to_update, channel, value, None);
}

fn check_update_bool_with_post_update(
    response: &egui::Response,
    channels_to_update: &mut Vec<ChannelUpdate>,
    channel: &Channel,
    value: bool,
    post_update: Option<bool>,
) {
    if response.changed() {
        channels_to_update.push(ChannelUpdate::update_bool_with_post(
            channel,
            value,
            post_update,
        ));
    }
}

fn update_bool(channels_to_update: &mut Vec<ChannelUpdate>, channel: &Channel, value: bool) {
    update_bool_with_post_update(channels_to_update, channel, value, None);
}
fn update_bool_with_post_update(
    channels_to_update: &mut Vec<ChannelUpdate>,
    channel: &Channel,
    value: bool,
    post_update: Option<bool>,
) {
    channels_to_update.push(ChannelUpdate::update_bool_with_post(
        channel,
        value,
        post_update,
    ));
}

impl From<Rect> for egui::Rect {
    fn from(value: Rect) -> Self {
        egui::Rect::from_min_size(
            egui::pos2(value.x.0, value.y.0),
            egui::vec2(value.width.0, value.height.0),
        )
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

/*
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
*/
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
