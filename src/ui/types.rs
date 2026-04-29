use core::f32;
use std::collections::HashMap;

pub type Ui = Layout<PrimUi>;
pub type UiRect = LayoutRect<PrimUi>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: Float,
    pub y: Float,
    pub width: Float,
    pub height: Float,
}

impl Rect {
    pub fn unit() -> Self {
        Rect {
            x: Float(0.0),
            y: Float(0.0),
            width: Float(1.0),
            height: Float(1.0),
        }
    }
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
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub color: Option<Col>,
    pub background: Option<Col>,
    pub pad: Option<Pad>,
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

pub fn get_root_rect(config: &UiConfig) -> Rect {
    Rect {
        x: Float(0.0),
        y: Float(0.0),
        width: config.config.size.width,
        height: config.config.size.height,
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
            let segments = split_line(&root.x, &root.width, proportions);
            segments
                .iter()
                .map(|(segment_start, segment_width)| Rect {
                    x: *segment_start,
                    width: *segment_width,
                    y: root.y,
                    height: root.height,
                })
                .collect()
        }
        Direction::Ver => {
            let segments = split_line(&root.y, &root.height, proportions);
            segments
                .iter()
                .map(|(segment_start, segment_height)| Rect {
                    y: *segment_start,
                    height: *segment_height,
                    x: root.x,
                    width: root.width,
                })
                .collect()
        }
    }
}

fn split_line(start: &Float, length: &Float, proportions: &[Float]) -> Vec<(Float, Float)> {
    proportions
        .iter()
        .scan(start.0, |state, x| {
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
        Layout::Hor { items } => items.scale,
        Layout::Ver { items } => items.scale,
        Layout::Prim { value } => value.scale,
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
        rect: *root,
    }
}

fn apply_rects(rects: &Vec<Rect>, items: &[Ui]) -> Vec<UiRect> {
    items
        .iter()
        .zip(rects)
        .map(|(item, rect)| get_ui_rect(rect, item))
        .collect()
}

fn set_widget_rect<T: Clone>(rect: &Rect, widget: &Widget<T>) -> WidgetRect<T> {
    WidgetRect {
        item: widget.item.clone(),
        style: widget.style.clone(),
        rect: *rect,
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub channels: ChannelMap,
    pub prims: Vec<WidgetRect<PrimUi>>,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelMap {
    pub floats: HashMap<Channel, f32>,
    pub bools: HashMap<Channel, bool>,
}

impl ChannelMap {
    pub fn get_mut_float(&mut self, channel: &Channel) -> &mut f32 {
        self.floats.get_mut(channel).unwrap()
    }

    pub fn get_mut_bool(&mut self, channel: &Channel) -> &mut bool {
        self.bools.get_mut(channel).unwrap()
    }

    pub fn get_all_channels(&self) -> Vec<Channel> {
        let mut res: Vec<Channel> = self.floats.keys().cloned().collect();
        res.extend(self.bools.keys().cloned().collect::<Vec<Channel>>());
        res
    }
}

pub fn get_ui_state(ui: &UiRect) -> UiState {
    let mut res = UiState::default();
    collect_channel_inits(&mut res, ui);
    res
}

pub fn collect_channel_inits(res: &mut UiState, ui: &UiRect) {
    match ui {
        LayoutRect::Hor { items } => items
            .item
            .iter()
            .for_each(|x| collect_channel_inits(res, x)),
        LayoutRect::Ver { items } => items
            .item
            .iter()
            .for_each(|x| collect_channel_inits(res, x)),
        LayoutRect::Prim { value } => {
            res.prims.push(value.clone());
            get_prim_ui_channels(&value.item)
                .iter()
                .for_each(|chan_by_type| match chan_by_type {
                    ChannelByType::FloatChannel(chan) => {
                        res.channels.floats.insert(chan.clone(), 0.0);
                    }
                    ChannelByType::BoolChannel(chan) => {
                        res.channels.bools.insert(chan.clone(), false);
                    }
                })
        }
    }
}

pub fn get_prim_ui_name(ui: &PrimUi) -> Option<String> {
    get_prim_ui_channels(ui)
        .first()
        .map(|x| x.get_channel().clone().0)
}

pub enum ChannelByType {
    FloatChannel(Channel),
    BoolChannel(Channel),
}

impl ChannelByType {
    pub fn get_channel(&self) -> Channel {
        match self {
            ChannelByType::FloatChannel(chan) => chan,
            ChannelByType::BoolChannel(chan) => chan,
        }
        .clone()
    }
}

pub fn get_prim_ui_channels(ui: &PrimUi) -> Vec<ChannelByType> {
    match ui {
        PrimUi::Knob { channel } => vec![ChannelByType::FloatChannel(channel.clone())],
        PrimUi::Slider { channel } => vec![ChannelByType::FloatChannel(channel.clone())],
        PrimUi::Label { text: _, size: _ } => vec![],
        PrimUi::Button { channel, text: _ } => vec![ChannelByType::BoolChannel(channel.clone())],
        PrimUi::Toggle { channel, text: _ } => vec![ChannelByType::BoolChannel(channel.clone())],
        PrimUi::Select { channel, text: _ } => vec![ChannelByType::FloatChannel(channel.clone())],
        PrimUi::Space => vec![],
        PrimUi::Image { file: _ } => vec![],
    }
}
