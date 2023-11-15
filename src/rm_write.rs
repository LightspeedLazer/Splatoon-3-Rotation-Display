use std::fmt::Display;
extern crate serde;
use self::serde::{Deserialize, Serialize};

#[allow(unused)]
pub fn write_to_skin(skin_path: &str, contents: Vec<RmObject>) -> Result<(), std::io::Error>{
    std::fs::write(skin_path, {
        let mut ret = String::from("[Rainmeter]\nUpdate=1000\nAccurateText=1\nContextTitle=Refresh File\nContextAction=[!CommandMeasure \"SplatinkCore\" \"RefreshFile\"]\nContextTitle2=Repull Data\nContextAction2=[!CommandMeasure \"SplatinkCore\" \"RepullData\"]\n[Metadata]\nName=Splatoon 3 Rotation Display\nAuthor=gamingtime\nInformation=Displays the future Splatoon 3 schedules along with upcoming and recent Splatfest data\nVersion=1.1.3\nLicense=Creative Commons Attribution - Non - Commercial - Share Alike 3.0\n");
        for obj in contents {
            ret += &format!("{obj}\n");
        }
        ret
    })
}

pub trait ToRM {
    fn to_rm(&self) -> Vec<RmObject>;
}

#[derive(Clone)]
pub struct Coord {
    pub x: isize,
    pub y: isize
}
impl From<(isize, isize)> for Coord {
    fn from(value: (isize, isize)) -> Self {
        Coord {
            x: value.0,
            y: value.1
        }
    }
}
impl std::ops::Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        (self.x+rhs.x, self.y+rhs.y).into()
    }
}
impl std::ops::AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
#[derive(PartialEq, Clone, Deserialize, Serialize)]
pub struct Color {
    r: isize,
    g: isize,
    b: isize,
    a: isize,
}
impl From<(isize, isize, isize, isize)> for Color {
    fn from(value: (isize, isize, isize, isize)) -> Self {
        Color {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{},{},{},{}",
            self.r,self.g,self.b,self.a
        )
    }
}

pub struct RmObject {
    pub name: String,
    pub object_type: ObjectType
}
impl Display for RmObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]\n{}", self.name, self.object_type)
    }
}
impl RmObject {
    pub fn new(object_type: ObjectType) -> Self {
        RmObject {
            name: String::new(),
            object_type
        }
    }
    pub fn prefix_name_owned(mut self, prefix: &str) -> Self {
        self.name = format!("{prefix}{}", self.name);
        if let ObjectType::Meter(_, ref mut o) = self.object_type {
            if let Some(n) = &o.measure_name {
                o.measure_name = Some(format!("{prefix}{n}"));
            }
        }
        self
    }
    pub fn prefix_name_mut(&mut self, prefix: &str) -> &mut Self {
        self.name = format!("{prefix}{}", self.name);
        if let ObjectType::Meter(_, ref mut o) = self.object_type {
            if let Some(n) = &o.measure_name {
                o.measure_name = Some(format!("{prefix}{n}"));
            }
        }
        self
    }
}

pub enum ObjectType {
    Measure(MeasureType, MeasureOptions),
    Meter(MeterType, MeterOptions),
}
impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Measure(t, o) => write!(f, "Measure={t}\n{o}"),
            ObjectType::Meter(t, o) => write!(f, "Meter={t}\n{o}"),
        }
    }
}

#[derive(Default)]
pub struct MeasureOptions {
    pub on_change_action: Vec<String>,
    pub dynamic_variables: bool,
}
impl Display for MeasureOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();
        if !self.on_change_action.is_empty() {
            ret += "OnChangeAction=";
            for action in &self.on_change_action {
                ret += &format!("[{action}]");
            }
            ret += "\n";
        }
        if self.dynamic_variables {
            ret += "DynamicVariables=1\n";
        }
        write!(f, "{ret}")
    }
}

pub enum MeasureType {
    Plugin(PluginType),
    String(String),
}
impl Display for MeasureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeasureType::Plugin(t) => write!(f, "Plugin\nPlugin={t}"),
            MeasureType::String(o) => write!(f, "String\n{o}"),
        }
    }
}

pub enum PluginType {
    Splatink(SplatinkType),
}
impl Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Splatink(t) => write!(f, "Splatink\nType={t}"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum SplatinkType {
    Core(String),
    TimeBar(TimeBarOptions),
}
impl Display for SplatinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplatinkType::Core(s) => write!(f, "Core\nSche={s}"),
            SplatinkType::TimeBar(o) => write!(f, "TimeBar\n{o}"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TimeBarOptions {
    pub start_time: i64,
    pub end_time: i64,
}
impl Display for TimeBarOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StartTime={}\nEndTime={}", self.start_time, self.end_time)
    }
}

pub struct MeterOptions {
    pub pos: Coord,
    pub size: Coord,
    pub solid_color: Option<Color>,
    pub measure_name: Option<String>,
    pub groups: Vec<String>,
    pub left_click_action: Vec<String>,
    pub right_click_action: Vec<String>,
    pub scroll_down_action: Vec<String>,
    pub scroll_up_action: Vec<String>,
    pub tool_tip: Option<ToolTip>,
    pub hidden: bool,
}
impl Display for MeterOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = format!("X={}\nY={}\nW={}\nH={}", self.pos.x, self.pos.y, self.size.x, self.size.y);
        if let Some(c) = &self.solid_color {
            ret += &format!("\nSolidColor={c}");
        }
        if let Some(m) = &self.measure_name {
            ret += &format!("\nMeasureName={m}");
        }
        if !self.groups.is_empty() {
            ret += &format!("\nGroup=");
            for group in &self.groups {
                ret += group;
                ret += "|";
            }
            ret.remove(ret.len() - 1);
        }
        if !self.left_click_action.is_empty() {
            ret += "\nLeftMouseUpAction=";
            for action in &self.left_click_action {
                ret += &format!("[{action}]");
            }
        }
        if !self.right_click_action.is_empty() {
            ret += "\nRightMouseUpAction=";
            for action in &self.right_click_action {
                ret += &format!("[{action}]");
            }
        }
        if !self.scroll_down_action.is_empty() {
            ret += "\nMouseScrollDownAction=";
            for action in &self.scroll_down_action {
                ret += &format!("[{action}]");
            }
        }
        if !self.scroll_up_action.is_empty() {
            ret += "\nMouseScrollUpAction=";
            for action in &self.scroll_up_action {
                ret += &format!("[{action}]");
            }
        }
        if let Some(t) = &self.tool_tip {
            ret += &format!("\n{t}");
        }
        if self.hidden {
            ret += &format!("\nHidden=1");
        }
        write!(f, "{ret}")
    }
}
impl MeterOptions {
    pub fn new() -> MeterOptions {
        MeterOptions {
            pos: (0,0).into(),
            size: (0,0).into(),
            solid_color: None,
            measure_name: None,
            groups: Vec::new(),
            left_click_action: Vec::new(),
            right_click_action: Vec::new(),
            scroll_down_action: Vec::new(),
            scroll_up_action: Vec::new(),
            tool_tip: None,
            hidden: false,
        }
    }
}

pub struct ToolTip {
    tool_tip_text: String,
}
impl Display for ToolTip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ToolTipText={}", self.tool_tip_text)
    }
}
impl ToolTip {
    pub fn new(tool_tip_text: String) -> ToolTip {
        ToolTip {
            tool_tip_text
        }
    }
}

pub enum MeterType {
    Image(ImageOptions),
    String(StringOptions),
    Bar(BarOptions),
}
impl Display for MeterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeterType::Image(o) => write!(f, "Image\n{o}"),
            MeterType::String(o) => write!(f, "String\n{o}"),
            MeterType::Bar(o) => write!(f, "Bar\n{o}"),
        }
    }
}

pub struct ImageOptions {
    pub image_name: String,
    pub preseve_aspect_ratio: bool,
}
impl Display for ImageOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = format!("ImageName={}", self.image_name);
        if self.preseve_aspect_ratio {
            ret.push_str(&format!("\nPreserveAspectRatio=1"))
        }
        write!(f, "{ret}")
    }
}

pub struct StringOptions {
    pub text: String,
    pub string_align: Option<StringAlign>,
    pub font_color: Option<Color>,
    pub font_size: Option<f64>,
    pub font_weight: Option<usize>,
    pub clip_string: Option<usize>,
    pub clip_string_w: Option<usize>,
    pub anti_alias: bool,
}
impl Display for StringOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = format!("Text={}", self.text);
        if let Some(x) = &self.string_align {
            ret += &format!("\nStringAlign={x}");
        }
        if let Some(x) = &self.font_color {
            ret += &format!("\nFontColor={x}");
        }
        if let Some(x) = &self.font_size {
            ret += &format!("\nFontSize={x}");
        }
        if let Some(x) = &self.font_weight {
            ret += &format!("\nFontWeight={x}");
        }
        if let Some(x) = &self.clip_string {
            ret += &format!("\nClipString={x}");
        }
        if let Some(x) = &self.clip_string_w {
            ret += &format!("\nClipStringW={x}");
        }
        if self.anti_alias {
            ret += &format!("\nAntiAlias=1");
        }
        write!(f, "{ret}")
    }
}
impl Default for StringOptions {
    fn default() -> Self {
        StringOptions {
            text: String::new(),
            string_align: Some(StringAlign::CenterCenter),
            font_color: Some((255,255,255,255).into()),
            font_size: Some(12_f64),
            font_weight: Some(800),
            clip_string: Some(2),
            clip_string_w: None,
            anti_alias: true,
        }
    }
}

#[derive(PartialEq)]
pub enum StringAlign {
    CenterCenter,
}
impl Display for StringAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringAlign::CenterCenter => write!(f, "CenterCenter"),
        }
    }
}

pub struct BarOptions {
    pub bar_color: Color,
    pub bar_orientation: BarOrientation,
}
impl Display for BarOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BarColor={}\nBarOrientation={}", self.bar_color, self.bar_orientation)
    }
}
pub enum BarOrientation {
    Vertical,
    Horizontal,
}
impl Display for BarOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarOrientation::Vertical => write!(f, "Vertical"),
            BarOrientation::Horizontal => write!(f, "Horizontal"),
        }
    }
}
