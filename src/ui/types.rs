use core::f32;
use std::collections::{HashMap, hash_map::IntoValues};

pub type Ui = Layout<PrimUi>;
pub type UiRect = LayoutRect<PrimUi>;

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
pub struct WidgetRect<T> {
    pub item: T,
    pub rect: Rect,
    pub style: Option<Style>,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: Float,
    pub y: Float,
    pub width: Float,
    pub height: Float,
}

#[derive(Debug, Clone)]
pub enum Layout<T> {
    Hor { items: Widget<Vec<Layout<T>>> },
    Ver { items: Widget<Vec<Layout<T>>> },
    Prim { value: Widget<T> },
}

#[derive(Debug, Clone)]
pub enum LayoutRect<T> {
    Hor {
        items: WidgetRect<Vec<LayoutRect<T>>>,
    },
    Ver {
        items: WidgetRect<Vec<LayoutRect<T>>>,
    },
    Prim {
        value: WidgetRect<T>,
    },
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
    Keyboard {}
    RadioButton {}
    Range {}
    ButtonGrid {}
    Dot {}
    Tabs {}
    PresetList {}

    Meter {}
    Spectrum {}
    Hist {}
    Wave {}
    */
}

pub fn get_ui_rect(root: &Rect, ui: &Ui) -> UiRect {
    match ui {
        Layout::Hor { items } => hor_rect(root, items),
        Layout::Ver { items } => ver_rect(root, items),
        Layout::Prim { value } => LayoutRect::Prim {
            value: prim_rect(root, value),
        },
    }
}

enum Direction {
    Hor,
    Ver,
}

fn direct_cons(direction: &Direction, items: WidgetRect<Vec<UiRect>>) -> UiRect {
    match direction {
        Direction::Hor => LayoutRect::Hor { items },
        Direction::Ver => LayoutRect::Ver { items },
    }
}

fn direct_split_rect(direction: &Direction, root: &Rect, proportions: &[Float]) -> Vec<Rect> {
    match direction {
        Direction::Hor => {
            let segments = split_line(&root.width, proportions);
            segments
                .iter()
                .map(|(segment_start, segment_width)| Rect {
                    x: segment_start.clone(),
                    width: segment_width.clone(),
                    y: root.y.clone(),
                    height: root.height.clone(),
                })
                .collect()
        }
        Direction::Ver => {
            let segments = split_line(&root.height, proportions);
            segments
                .iter()
                .map(|(segment_start, segment_height)| Rect {
                    y: segment_start.clone(),
                    height: segment_height.clone(),
                    x: root.x.clone(),
                    width: root.width.clone(),
                })
                .collect()
        }
    }
}

fn split_line(length: &Float, proportions: &[Float]) -> Vec<(Float, Float)> {
    proportions
        .iter()
        .scan(0.0, |state, x| {
            let segment_length = length.0 * x.0;
            let res = (Float(*state), Float(segment_length));
            *state += segment_length;
            Some(res)
        })
        .collect()
}

fn hor_rect(rect: &Rect, items: &Widget<Vec<Ui>>) -> UiRect {
    group_rect_by(&Direction::Hor, rect, items)
}

fn ver_rect(rect: &Rect, items: &Widget<Vec<Ui>>) -> UiRect {
    group_rect_by(&Direction::Ver, rect, items)
}

fn group_rect_by(direction: &Direction, rect: &Rect, items: &Widget<Vec<Ui>>) -> UiRect {
    let proportions: Vec<Float> =
        get_proportions(&items.item.iter().map(get_scale).collect::<Vec<Float>>());
    let rects = direct_split_rect(direction, rect, &proportions);
    direct_cons(direction, apply_rects_over_widget(rect, &rects, items))
}

fn prim_rect(rect: &Rect, item: &Widget<PrimUi>) -> WidgetRect<PrimUi> {
    set_widget_rect(rect, item)
}

fn get_scale(item: &Ui) -> Float {
    match item {
        Layout::Hor { items } => items.scale.clone(),
        Layout::Ver { items } => items.scale.clone(),
        Layout::Prim { value } => value.scale.clone(),
    }
    .unwrap_or(Float(1.0))
}

fn get_proportions(values: &[Float]) -> Vec<Float> {
    let total: f32 = values.iter().map(|x| x.0.abs()).sum();
    if total > f32::EPSILON {
        values.iter().map(|x| Float(x.0.abs() / total)).collect()
    } else {
        values.to_vec()
    }
}

fn apply_rects_over_widget(
    root: &Rect,
    rects: &Vec<Rect>,
    items: &Widget<Vec<Ui>>,
) -> WidgetRect<Vec<UiRect>> {
    WidgetRect {
        item: apply_rects(rects, &items.item),
        style: items.style.clone(),
        rect: root.clone(),
    }
}

fn apply_rects(rects: &Vec<Rect>, items: &Vec<Ui>) -> Vec<UiRect> {
    items
        .iter()
        .zip(rects)
        .map(|(item, rect)| set_widget_rect_layout(rect, item))
        .collect()
}

fn set_widget_rect_layout(rect: &Rect, widget: &Ui) -> UiRect {
    match widget {
        Layout::Hor { items } => {
            let rec = set_widget_rect(rect, items);
            LayoutRect::Hor {
                items: WidgetRect {
                    item: rec
                        .item
                        .iter()
                        .map(|x| get_ui_rect(rect, x))
                        .collect::<Vec<UiRect>>(),
                    style: rec.style.clone(),
                    rect: rec.rect.clone(),
                },
            }
        }

        Layout::Ver { items } => {
            let rec = set_widget_rect(rect, items);
            LayoutRect::Ver {
                items: WidgetRect {
                    item: rec
                        .item
                        .iter()
                        .map(|x| get_ui_rect(rect, x))
                        .collect::<Vec<UiRect>>(),
                    style: rec.style.clone(),
                    rect: rec.rect.clone(),
                },
            }
        }
        Layout::Prim { value } => LayoutRect::Prim {
            value: prim_rect(rect, value),
        },
    }
}

fn set_widget_rect<T: Clone>(rect: &Rect, widget: &Widget<T>) -> WidgetRect<T> {
    WidgetRect {
        item: widget.item.clone(),
        style: widget.style.clone(),
        rect: rect.clone(),
    }
}
