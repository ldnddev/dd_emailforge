use serde::{Deserialize, Serialize};

fn default_lang() -> String {
    "en".to_string()
}
fn default_font() -> String {
    "Arial, Helvetica, sans-serif".to_string()
}
fn default_text_color() -> String {
    "#1a1a1a".to_string()
}
fn default_bg() -> String {
    "#f4f4f4".to_string()
}
fn default_width() -> u32 {
    600
}
fn default_btn_bg() -> String {
    "#FFAF46".to_string()
}
fn default_btn_fg() -> String {
    "#0F1114".to_string()
}
fn default_breakpoint() -> String {
    "480px".to_string()
}
fn default_logo_width() -> String {
    "160px".to_string()
}
fn default_spacer_height() -> String {
    "24px".to_string()
}
fn default_icon_size() -> String {
    "32px".to_string()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Template {
    pub version: u32,
    pub name: String,
    pub subject: String,
    #[serde(default)]
    pub preheader: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub dir: String,
    #[serde(default)]
    pub base_url: String,
    pub brand: Brand,
    pub head: Head,
    pub body: Body,
}

impl Template {
    #[allow(dead_code)]
    pub fn minimal() -> Self {
        Self {
            version: 1,
            name: "welcome".to_string(),
            subject: "You're in.".to_string(),
            preheader: "Here's what happens next.".to_string(),
            lang: default_lang(),
            dir: String::new(),
            base_url: String::new(),
            brand: Brand::default(),
            head: Head {
                title: "You're in.".to_string(),
                ..Head::default()
            },
            body: Body::default(),
        }
    }

    pub fn normalize_base_url(&mut self) {
        let trimmed = self.base_url.trim();
        if trimmed.is_empty() {
            self.base_url.clear();
            return;
        }
        if trimmed.ends_with('/') {
            self.base_url = trimmed.to_string();
        } else {
            self.base_url = format!("{trimmed}/");
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Brand {
    #[serde(default = "default_font")]
    pub font_family: String,
    #[serde(default = "default_text_color")]
    pub text_color: String,
    #[serde(default = "default_bg")]
    pub background_color: String,
    #[serde(default = "default_width")]
    pub content_width: u32,
    #[serde(default = "default_btn_bg")]
    pub button_background: String,
    #[serde(default = "default_btn_fg")]
    pub button_color: String,
}

impl Default for Brand {
    fn default() -> Self {
        Self {
            font_family: default_font(),
            text_color: default_text_color(),
            background_color: default_bg(),
            content_width: default_width(),
            button_background: default_btn_bg(),
            button_color: default_btn_fg(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Head {
    pub title: String,
    #[serde(default = "default_breakpoint")]
    pub breakpoint: String,
    #[serde(default)]
    pub fonts: Vec<WebFont>,
    #[serde(default)]
    pub json_ld: String,
    #[serde(default)]
    pub css: String,
    #[serde(default)]
    pub css_inline: bool,
}

impl Default for Head {
    fn default() -> Self {
        Self {
            title: String::new(),
            breakpoint: default_breakpoint(),
            fonts: Vec::new(),
            json_ld: String::new(),
            css: String::new(),
            css_inline: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebFont {
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Body {
    #[serde(default)]
    pub background_color: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub nodes: Vec<BodyNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BodyNode {
    MjSection(MjSection),
    MjWrapper(MjWrapper),
    MjHero(MjHero),
    EmailHeader(EmailHeader),
    EmailHero(EmailHero),
    EmailCta(EmailCta),
    EmailArticle(EmailArticle),
    EmailFooter(EmailFooter),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SectionChild {
    MjColumn(MjColumn),
    MjGroup(MjGroup),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ColumnChild {
    MjText(MjText),
    MjButton(MjButton),
    MjImage(MjImage),
    MjDivider(MjDivider),
    MjSpacer(MjSpacer),
    MjSocial(MjSocial),
    MjTable(MjTable),
    MjNavbar(MjNavbar),
    MjAccordion(MjAccordion),
    MjCarousel(MjCarousel),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HeroMode {
    #[default]
    FluidHeight,
    FixedHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SocialMode {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Thumbnails {
    Visible,
    #[default]
    Hidden,
    Supported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ImagePosition {
    #[default]
    Top,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SocialNetwork {
    Facebook,
    Instagram,
    Linkedin,
    X,
    Github,
    Web,
    Youtube,
    Pinterest,
    Google,
    Tumblr,
    Snapchat,
    Vimeo,
    Medium,
    Soundcloud,
    Dribbble,
    Xing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gutter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_repeat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(default)]
    pub full_width: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub children: Vec<SectionChild>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjColumn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub components: Vec<ColumnChild>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjWrapper {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_repeat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(default)]
    pub full_width: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub children: Vec<BodyNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub children: Vec<MjColumn>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjText {
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjButton {
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MjImage {
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default = "default_true")]
    pub fluid_on_mobile: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

impl Default for MjImage {
    fn default() -> Self {
        Self {
            src: String::new(),
            alt: String::new(),
            href: None,
            width: None,
            height: None,
            align: None,
            fluid_on_mobile: true,
            border: None,
            border_radius: None,
            title: None,
            padding: None,
            target: None,
            rel: None,
            css_class: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjDivider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MjSpacer {
    #[serde(default = "default_spacer_height")]
    pub height: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

impl Default for MjSpacer {
    fn default() -> Self {
        Self {
            height: default_spacer_height(),
            padding: None,
            css_class: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MjSocial {
    #[serde(default)]
    pub mode: SocialMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default = "default_icon_size")]
    pub icon_size: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub elements: Vec<MjSocialElement>,
}

impl Default for MjSocial {
    fn default() -> Self {
        Self {
            mode: SocialMode::Horizontal,
            align: None,
            icon_size: default_icon_size(),
            border_radius: None,
            padding: None,
            icon_padding: None,
            inner_padding: None,
            font_size: None,
            color: None,
            css_class: None,
            elements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MjSocialElement {
    pub name: SocialNetwork,
    pub href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

impl Default for MjSocialElement {
    fn default() -> Self {
        Self {
            name: SocialNetwork::X,
            href: String::new(),
            src: None,
            background_color: None,
            padding: None,
            icon_size: None,
            alt: None,
            css_class: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjTable {
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cellpadding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cellspacing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjNavbar {
    #[serde(default)]
    pub hamburger: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ico_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ico_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ico_font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ico_padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ico_open: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ico_close: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub links: Vec<MjNavbarLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjNavbarLink {
    #[serde(default)]
    pub href: String,
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjAccordion {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_wrapped_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_unwrapped_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub elements: Vec<MjAccordionElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjAccordionElement {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjCarousel {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tb_border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub thumbnails: Thumbnails,
    #[serde(default)]
    pub images: Vec<MjCarouselImage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjCarouselImage {
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnails_src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MjHero {
    #[serde(default)]
    pub mode: HeroMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_padding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_class: Option<String>,
    #[serde(default)]
    pub children: Vec<ColumnChild>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailHeader {
    #[serde(default)]
    pub logo_src: String,
    #[serde(default)]
    pub logo_alt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_href: Option<String>,
    #[serde(default = "default_logo_width")]
    pub logo_width: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailHero {
    #[serde(default)]
    pub image_src: String,
    #[serde(default)]
    pub image_alt: String,
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub subheading: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailCta {
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub copy: String,
    #[serde(default)]
    pub button_label: String,
    #[serde(default)]
    pub button_href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailArticle {
    #[serde(default)]
    pub image_src: String,
    #[serde(default)]
    pub image_alt: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub copy: String,
    #[serde(default)]
    pub link_label: String,
    #[serde(default)]
    pub link_href: String,
    #[serde(default)]
    pub image_position: ImagePosition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailFooter {
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub address_lines: Vec<String>,
    #[serde(default)]
    pub unsubscribe_label: String,
    #[serde(default)]
    pub unsubscribe_href: String,
    #[serde(default)]
    pub social: Vec<MjSocialElement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kitchen_sink() -> Template {
        let mut t = Template::minimal();
        t.base_url = "https://cdn.example.com".to_string();
        t.head.fonts.push(WebFont {
            name: "Raleway".to_string(),
            href: "https://fonts.googleapis.com/css2?family=Raleway:wght@400;700&display=swap"
                .to_string(),
        });
        t.body.nodes = vec![
            BodyNode::EmailHeader(EmailHeader {
                logo_src: "images/logo.png".to_string(),
                logo_alt: "Logo".to_string(),
                logo_href: Some("https://example.com".to_string()),
                logo_width: "160px".to_string(),
                background_color: None,
            }),
            BodyNode::EmailHero(EmailHero {
                image_src: String::new(),
                image_alt: String::new(),
                heading: "Hi".to_string(),
                subheading: "There".to_string(),
                background_color: None,
            }),
            BodyNode::MjSection(MjSection {
                background_color: None,
                padding: None,
                full_width: false,
                children: vec![SectionChild::MjColumn(MjColumn {
                    width: Some("100%".to_string()),
                    background_color: None,
                    padding: None,
                    inner_background_color: None,
                    components: vec![
                        ColumnChild::MjText(MjText {
                            content: "Hello".to_string(),
                            align: Some(Align::Left),
                            font_size: None,
                            font_family: Some("Raleway, Arial, sans-serif".to_string()),
                            color: None,
                            padding: None,
                            ..Default::default()
                        }),
                        ColumnChild::MjButton(MjButton {
                            content: "Read more".to_string(),
                            href: "https://example.com".to_string(),
                            background_color: None,
                            color: None,
                            align: None,
                            font_family: None,
                            border_radius: None,
                            width: None,
                            padding: None,
                            ..Default::default()
                        }),
                        ColumnChild::MjImage(MjImage {
                            src: "https://cdn.example.com/a.png".to_string(),
                            alt: "A".to_string(),
                            href: None,
                            width: None,
                            align: Some(Align::Center),
                            fluid_on_mobile: true,
                            padding: None,
                            ..Default::default()
                        }),
                        ColumnChild::MjSocial(MjSocial {
                            mode: SocialMode::Horizontal,
                            align: None,
                            icon_size: "32px".to_string(),
                            elements: vec![MjSocialElement {
                                name: SocialNetwork::X,
                                href: "https://x.com/acme".to_string(),
                                src: None,
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ColumnChild::MjTable(MjTable {
                            content: "<tr><td>1</td></tr>".to_string(),
                            font_size: None,
                            color: None,
                            padding: None,
                            ..Default::default()
                        }),
                        ColumnChild::MjNavbar(MjNavbar {
                            hamburger: true,
                            ico_color: Some("#ffffff".to_string()),
                            base_url: None,
                            align: None,
                            padding: None,
                            links: vec![MjNavbarLink {
                                href: "https://example.com".to_string(),
                                content: "Home".to_string(),
                                color: None,
                                padding: None,
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ColumnChild::MjAccordion(MjAccordion {
                            border: None,
                            padding: None,
                            elements: vec![MjAccordionElement {
                                title: "Why?".to_string(),
                                content: "Because.".to_string(),
                                background_color: None,
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ColumnChild::MjCarousel(MjCarousel {
                            align: None,
                            padding: None,
                            thumbnails: Thumbnails::Hidden,
                            images: vec![MjCarouselImage {
                                src: "https://cdn.example.com/slide.png".to_string(),
                                alt: "Slide".to_string(),
                                href: None,
                                thumbnails_src: None,
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                })],
                ..Default::default()
            }),
            BodyNode::MjHero(MjHero {
                mode: HeroMode::FluidHeight,
                background_url: None,
                background_color: None,
                background_height: None,
                width: None,
                height: None,
                children: vec![],
                ..Default::default()
            }),
            BodyNode::EmailCta(EmailCta {
                heading: "Go".to_string(),
                copy: "Now".to_string(),
                button_label: "Shop".to_string(),
                button_href: "https://example.com".to_string(),
                background_color: None,
            }),
            BodyNode::EmailArticle(EmailArticle {
                image_src: String::new(),
                image_alt: String::new(),
                title: "Story".to_string(),
                copy: "Body".to_string(),
                link_label: "More".to_string(),
                link_href: "https://example.com".to_string(),
                image_position: ImagePosition::Left,
            }),
            BodyNode::EmailFooter(EmailFooter {
                company_name: "Acme".to_string(),
                address_lines: vec!["1 Main St".to_string()],
                unsubscribe_label: "Unsubscribe".to_string(),
                unsubscribe_href: "*|UNSUB|*".to_string(),
                social: vec![],
                copyright: Some("© Acme".to_string()),
            }),
        ];
        t
    }

    #[test]
    fn inner_enums_round_trip_kebab_case() {
        let json = serde_json::to_string(&HeroMode::FluidHeight).unwrap();
        assert_eq!(json, "\"fluid-height\"");
        let json = serde_json::to_string(&SocialMode::Horizontal).unwrap();
        assert_eq!(json, "\"horizontal\"");
        let json = serde_json::to_string(&ImagePosition::Top).unwrap();
        assert_eq!(json, "\"top\"");
        let json = serde_json::to_string(&SocialNetwork::X).unwrap();
        assert_eq!(json, "\"x\"");
        let json = serde_json::to_string(&Align::Center).unwrap();
        assert_eq!(json, "\"center\"");
        let json = serde_json::to_string(&Thumbnails::Hidden).unwrap();
        assert_eq!(json, "\"hidden\"");
        let json = serde_json::to_string(&Thumbnails::Supported).unwrap();
        assert_eq!(json, "\"supported\"");
    }

    #[test]
    fn body_node_type_tag_is_kebab_case() {
        let node = BodyNode::EmailHero(EmailHero {
            image_src: String::new(),
            image_alt: String::new(),
            heading: "H".to_string(),
            subheading: String::new(),
            background_color: None,
        });
        let v = serde_json::to_value(&node).unwrap();
        assert_eq!(v["type"], "email-hero");
    }

    #[test]
    fn extra_keys_are_ignored() {
        let raw = r#"{
            "version": 1,
            "name": "n",
            "subject": "s",
            "brand": {},
            "head": { "title": "t", "future": true },
            "body": { "nodes": [], "extra": 1 }
        }"#;
        let t: Template = serde_json::from_str(raw).unwrap();
        assert_eq!(t.head.title, "t");
        assert_eq!(t.lang, "en");
        assert_eq!(t.brand.content_width, 600);
    }

    #[test]
    fn kitchen_sink_round_trip() {
        let t = kitchen_sink();
        let json = serde_json::to_string_pretty(&t).unwrap();
        let back: Template = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
        assert!(json.contains("\"type\": \"mj-section\""));
        assert!(json.contains("\"mode\": \"fluid-height\""));
        assert!(json.contains("\"name\": \"x\""));
        assert!(json.contains("\"image_position\": \"left\""));
    }
}
