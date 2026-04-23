use crate::ui::types::*;
use hashlink::linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use yaml_rust2::YamlLoader;
use yaml_rust2::yaml::Yaml;

type ValueMap = LinkedHashMap<Yaml, Yaml>;

trait GetStr {
    fn get_str(&self, s: &str) -> Option<&Yaml>;
}

impl GetStr for Yaml {
    fn get_str(&self, s: &str) -> Option<&Yaml> {
        let map = self.as_hash()?;
        map.get(&Yaml::String(s.to_string()))
    }
}

impl From<yaml_rust2::ScanError> for Error {
    fn from(err: yaml_rust2::ScanError) -> Error {
        Error(err.to_string())
    }
}

type Value = Yaml;

#[derive(Debug, Clone)]
pub struct Error(pub String);

pub fn parse_config(input: &str) -> Result<UiConfig, Error> {
    let parsed = YamlLoader::load_from_str(input)?;
    UiConfig::try_from(parsed[0].clone())
}

impl TryFrom<Value> for UiConfig {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, missing_fields("UI config", "ui"))?;
        let config = parse_field_or_default("config", &map, "UI config");
        let state = parse_field_or_default("state", &map, "UI config");
        let ui = parse_field("ui", &map, "UI config")?;
        let csound = parse_field_or_default("csound", &map, "UI config");
        Ok(UiConfig {
            config,
            state,
            ui,
            csound,
        })
    }
}

impl TryFrom<Value> for Config {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("config"))?;
        let size = parse_field_or_default("size", &map, "config");
        Ok(Config { size })
    }
}

impl TryFrom<Value> for Size {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("size"))?;
        let width = parse_field("width", &map, "size")?;
        let height = parse_field("height", &map, "size")?;
        Ok(Size { width, height })
    }
}

impl TryFrom<Value> for Float {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Yaml::Real(f) => Ok(Float(f.parse::<f32>().unwrap())),
            Yaml::Integer(n) => Ok(Float(n as f32)),
            _ => Err(Error("Expected a number".to_string())),
        }
    }
}

impl TryFrom<Value> for Channel {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let name = value
            .as_str()
            .ok_or_else(|| Error("Value not a string".to_string()))?;
        Ok(Channel(name.to_string()))
    }
}

impl TryFrom<Value> for State {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("state"))?;
        let init = parse_field("init", &map, "state")?;
        Ok(State { init })
    }
}

impl TryFrom<Value> for Init {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("init"))?;
        let values = map
            .iter()
            .flat_map(
                |(key, val): (&Value, &Value)| -> Result<(Channel, Float), Error> {
                    let channel = Channel::try_from(key.clone())?;
                    let value = Float::try_from(val.clone())?;
                    Ok((channel, value))
                },
            )
            .collect::<HashMap<Channel, Float>>();
        Ok(Init { values })
    }
}

impl TryFrom<Value> for Ui {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("init"))?;
        let scale = get_scale(&value);
        let style = parse_field("style", &map, "UI widget").ok();
        if has_key(&map, "hor") {
            let values = value.get_str("hor").unwrap();
            let item = parse_ui_items(values)?;
            Ok(Layout::Hor {
                items: Widget { item, scale, style },
            })
        } else if has_key(&map, "ver") {
            let values = value.get_str("ver").unwrap();
            let item = parse_ui_items(values)?;
            Ok(Layout::Ver {
                items: Widget { item, scale, style },
            })
        } else {
            let item = PrimUi::try_from(value.clone())?;
            Ok(Layout::Prim {
                value: Widget { item, scale, style },
            })
        }
    }
}

impl TryFrom<Value> for PrimUi {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = value
            .as_hash()
            .ok_or_else(|| Error("Not a map".to_string()))?;
        if has_key(map, "knob") {
            parse_knob(&value)
        } else if has_key(map, "slider") {
            parse_slider(&value)
        } else if has_key(map, "toggle") {
            parse_toggle(&value)
        } else if has_key(map, "select") {
            parse_select(&value)
        } else if has_key(map, "image") {
            parse_image(&value)
        } else if has_key(map, "button") {
            parse_button(&value)
        } else if has_key(map, "space") {
            parse_space(&value)
        } else if has_key(map, "label") {
            parse_label(&value)
        } else {
            Err(Error(format!("Uknown widget: {:?}", &value)))
        }
    }
}

fn parse_channel(value: &Value, name: &str) -> Result<Channel, Error> {
    Channel::try_from(value.get_str(name).unwrap().clone())
}

fn parse_knob(value: &Value) -> Result<PrimUi, Error> {
    let channel = parse_channel(value, "knob")?;
    Ok(PrimUi::Knob { channel })
}

fn parse_slider(value: &Value) -> Result<PrimUi, Error> {
    let channel = parse_channel(value, "slider")?;
    Ok(PrimUi::Slider { channel })
}

fn parse_button(value: &Value) -> Result<PrimUi, Error> {
    let channel = parse_channel(value, "button")?;
    let text = get_text(value);
    Ok(PrimUi::Button { channel, text })
}

fn get_text(value: &Value) -> String {
    get_text_opt(value).unwrap_or("".to_string())
}

fn get_text_opt(value: &Value) -> Option<String> {
    let val = value.get_str("text")?;
    let str = val.as_str()?;
    Some(str.to_string())
}

fn parse_select(value: &Value) -> Result<PrimUi, Error> {
    let channel = parse_channel(value, "select")?;
    let text = parse_select_items(value)?;
    Ok(PrimUi::Select { channel, text })
}

fn parse_select_items(value: &Value) -> Result<Vec<String>, Error> {
    let values = value
        .get_str("text")
        .ok_or(missing_fields("select", "text"))?;
    let err = Error("Select text field is not a sequence of names".to_string());
    let items = values.as_vec().ok_or(err.clone())?;
    items
        .iter()
        .map(|x| x.as_str().map(|x| x.to_string()).ok_or(err.clone()))
        .collect::<Result<Vec<String>, Error>>()
}

fn parse_toggle(value: &Value) -> Result<PrimUi, Error> {
    let channel = parse_channel(value, "toggle")?;
    let text = get_text(value);
    Ok(PrimUi::Button { channel, text })
}

fn parse_label(value: &Value) -> Result<PrimUi, Error> {
    let text = get_text(value);
    let map = get_value_map(value, no_object("label"))?;
    let size = parse_field_or("size", &map, "label", Float(10.0));
    Ok(PrimUi::Label { text, size })
}

fn parse_space(_value: &Value) -> Result<PrimUi, Error> {
    Ok(PrimUi::Space)
}

fn parse_image(value: &Value) -> Result<PrimUi, Error> {
    let file_value = value
        .get_str("file")
        .ok_or(missing_fields("image", "file"))?;
    let file = file_value
        .as_str()
        .ok_or(Error("Image field file not a string".to_string()))?;
    Ok(PrimUi::Image {
        file: file.to_string(),
    })
}

fn parse_ui_items(value: &Value) -> Result<Vec<Layout<PrimUi>>, Error> {
    let values = value
        .as_vec()
        .ok_or_else(|| Error("expected a sequence".to_string()))?;
    values.iter().map(|x| Ui::try_from(x.clone())).collect()
}

fn has_key(map: &ValueMap, name: &str) -> bool {
    map.contains_key(&Value::String(name.to_string()))
}

fn get_scale(value: &Value) -> Option<Float> {
    if let Some(num) = value.get_str("scale") {
        Float::try_from(num.clone()).ok()
    } else {
        None
    }
}

impl TryFrom<Value> for Csound {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("csound"))?;
        let write: VecChannel = parse_field_or_default("write", &map, "csound");
        let read: VecChannel = parse_field_or_default("read", &map, "csound");
        Ok(Csound {
            write: write.0,
            read: read.0,
        })
    }
}

impl TryFrom<Value> for Style {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("init"))?;
        let color = parse_field("color", &map, "style").ok();
        let background = parse_field("background", &map, "style").ok();
        let pad = parse_field("pad", &map, "style").ok();
        Ok(Style {
            color,
            background,
            pad,
        })
    }
}

impl TryFrom<Value> for Pad {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let map = get_value_map(&value, no_object("init"))?;
        let left = parse_field_or("left", &map, "pad", Float(0.0));
        let right = parse_field_or("right", &map, "pad", Float(0.0));
        let bottom = parse_field_or("bottom", &map, "pad", Float(0.0));
        let top = parse_field_or("top", &map, "pad", Float(0.0));
        Ok(Pad {
            left,
            right,
            bottom,
            top,
        })
    }
}

impl TryFrom<Value> for Col {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Some(str) = value.as_str() {
            if is_hash_col(str) {
                Ok(Col::Hash {
                    hash: str.to_string(),
                })
            } else {
                Ok(Col::Name {
                    name: str.to_string(),
                })
            }
        } else {
            let map = get_value_map(
                &value,
                Error("Color should be string or object with RGB values".to_string()),
            )?;
            let r = parse_field("r", &map, "col")?;
            let g = parse_field("g", &map, "col")?;
            let b = parse_field("b", &map, "col")?;
            Ok(Col::Rgb { r, g, b })
        }
    }
}

fn is_hash_col(s: &str) -> bool {
    s.starts_with('#')
}

#[derive(Default)]
struct VecChannel(pub Vec<Channel>);

impl TryFrom<Value> for VecChannel {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let items = value
            .as_vec()
            .ok_or_else(|| Error("Not a sequence".to_string()))?;
        Ok(VecChannel(
            items
                .iter()
                .flat_map(|value: &Value| Channel::try_from(value.clone()))
                .collect(),
        ))
    }
}

fn parse_field<T: TryFrom<Value, Error = Error>>(
    field_name: &str,
    map: &ValueMap,
    type_name: &str,
) -> Result<T, Error> {
    let value = map
        .get(&Value::String(field_name.to_string()))
        .ok_or_else(|| missing_fields(type_name, field_name))?;
    T::try_from(value.clone())
}

fn parse_field_or_default<T: TryFrom<Value, Error = Error> + Default>(
    field_name: &str,
    map: &ValueMap,
    type_name: &str,
) -> T {
    parse_field(field_name, map, type_name).unwrap_or_default()
}

fn parse_field_or<T: TryFrom<Value, Error = Error>>(
    field_name: &str,
    map: &ValueMap,
    type_name: &str,
    def_value: T,
) -> T {
    parse_field(field_name, map, type_name).unwrap_or(def_value)
}

fn missing_fields_message(item: &str, fields: &str) -> String {
    format!("{} is missing fields: {}", item, fields)
}

fn missing_fields(item: &str, fields: &str) -> Error {
    Error(missing_fields_message(item, fields))
}

fn no_object(item: &str) -> Error {
    Error(format!("{} should be object", item))
}

fn get_value_map(value: &Value, msg: Error) -> Result<ValueMap, Error> {
    if let Yaml::Hash(map) = value {
        Ok(map.clone())
    } else {
        Err(msg)
    }
}
