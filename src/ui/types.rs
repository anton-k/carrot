use std::collections::HashMap;

pub type Ui = Layout<PrimUi>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Float(pub f32);

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub config: Config,
    pub state: State,
    pub csound: Csound,
    pub ui: Ui,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct Size {
    pub width: Float,
    pub height: Float,
}

impl Default for Size {
    fn default() -> Size {
        Size {
            width: Float(100.0),
            height: Float(100.0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Csound {
    pub write: Vec<Channel>,
    pub read: Vec<Channel>,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub init: Init,
}

#[derive(Debug, Clone, Default)]
pub struct Init {
    pub values: HashMap<Channel, Float>,
}

#[derive(Debug, Clone)]
pub struct Widget<T> {
    pub item: T,
    pub style: Option<Style>,
    pub scale: Option<Float>,
}

#[derive(Debug, Clone)]
pub enum Layout<T> {
    Hor { items: Widget<Vec<Layout<T>>> },
    Ver { items: Widget<Vec<Layout<T>>> },
    Prim { value: Widget<T> },
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Col {
    Rgb { r: Float, g: Float, b: Float },
    Hash { hash: String },
    Name { name: String },
}
#[derive(Debug, Clone)]
pub struct Style {
    pub color: Option<Col>,
    pub background: Option<Col>,
    pub pad: Option<Pad>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            color: None,
            background: None,
            pad: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pad {
    pub left: Float,
    pub right: Float,
    pub bottom: Float,
    pub top: Float,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Channel(pub String);

#[derive(Debug, Clone)]
pub enum PrimUi {
    Knob { channel: Channel },
    Slider { channel: Channel },
    Label { text: String, size: Float },
    Button { channel: Channel, text: String },
    Toggle { channel: Channel, text: String },
    Select { channel: Channel, text: Vec<String> },
    Space,
    Image { file: String },
    /*
    XYPad {
        channel: (Channel, Channel),
    },
    */
}
