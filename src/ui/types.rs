pub type Ui = Layout<PrimUi>;

#[derive(Debug, Clone)]
pub struct Scale<T> {
    scale: f32,
    value: T,
}

#[derive(Debug, Clone)]
pub enum Layout<T> {
    Hor { items: Vec<Scale<Layout<T>>> },
    Ver { items: Vec<Scale<Layout<T>>> },
    Group { items: Box<Layout<T>>, style: Style },
    Prim { value: T },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Col {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct Style {
    color: Col,
    background: Col,
    pad: Pad,
}

#[derive(Debug, Clone)]
pub struct Pad {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Channel(String);

#[derive(Debug, Clone)]
pub enum PrimUi {
    Knob {
        channel: Channel,
        range: (f64, f64),
        value: f64,
        style: Style,
    },
    Slider {
        channel: Channel,
        range: (f64, f64),
        value: f64,
        style: Style,
    },
    Text {
        value: String,
        size: f32,
        style: Style,
    },
    Button {
        channel: Channel,
        label: String,
        style: Style,
    },
    Toggle {
        channel: Channel,
        label: String,
        value: bool,
        style: Style,
    },
    Select {
        labels: Vec<String>,
        style: Style,
        value: u16,
    },
    Space,
    /*

    Image {
        file: String,
        style: Style,
    },
    XYPad {
        channel: (Channel, Channel),
        range: ((f64, f64), (f64, f64)),
        value: (f64, f64),
        style: Style,
    },
    */
}

pub struct Error(String);

pub fn parse_ui() -> Result<Ui, Error> {
    Ok(Layout::Hor {
        items: vec![Scale {
            scale: 1.0,
            value: Layout::Prim {
                value: PrimUi::Space,
            },
        }],
    })
}
