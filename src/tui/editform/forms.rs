//! Static FormEdit maps. Field ids match JSON / model names.
use super::*;

const fn f(id: &'static str, label: &'static str, kind: FieldKind, required: bool) -> FormField {
    FormField {
        id,
        label,
        kind,
        required,
        visible_when: None,
        hint: None,
        placeholder: None,
        brand_placeholder: None,
    }
}

const fn with_brand(mut field: FormField, token: BrandPlaceholder) -> FormField {
    field.brand_placeholder = Some(token);
    if field.hint.is_none() {
        field.hint = Some("empty uses brand");
    }
    field
}

const fn font_family_field() -> FormField {
    with_brand(
        f(
            "font_family",
            "Font family",
            FieldKind::Text { default: "" },
            false,
        ),
        BrandPlaceholder::FontFamily,
    )
}

const fn text_color_field() -> FormField {
    with_brand(
        f("color", "Color", FieldKind::Text { default: "" }, false),
        BrandPlaceholder::TextColor,
    )
}

const fn button_color_field() -> FormField {
    with_brand(
        f("color", "Color", FieldKind::Text { default: "" }, false),
        BrandPlaceholder::ButtonColor,
    )
}

const fn button_bg_field() -> FormField {
    with_brand(
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        BrandPlaceholder::ButtonBackground,
    )
}

const fn body_bg_field() -> FormField {
    with_brand(
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        BrandPlaceholder::Background,
    )
}

const fn padding_field() -> FormField {
    FormField {
        id: "padding",
        label: "Padding",
        kind: FieldKind::Text { default: "" },
        required: false,
        visible_when: None,
        hint: Some(crate::padding::HINT),
        placeholder: Some(crate::padding::PLACEHOLDER),
        brand_placeholder: None,
    }
}

const fn inner_padding_field() -> FormField {
    FormField {
        id: "inner_padding",
        label: "Inner padding",
        kind: FieldKind::Text { default: "" },
        required: false,
        visible_when: None,
        hint: Some(crate::padding::HINT),
        placeholder: Some("e.g. 12px 24px"),
        brand_placeholder: None,
    }
}

const fn hinted(
    id: &'static str,
    label: &'static str,
    hint: &'static str,
    placeholder: &'static str,
) -> FormField {
    FormField {
        id,
        label,
        kind: FieldKind::Text { default: "" },
        required: false,
        visible_when: None,
        hint: Some(hint),
        placeholder: Some(placeholder),
        brand_placeholder: None,
    }
}

const fn border_field() -> FormField {
    FormField {
        id: "border",
        label: "Border",
        kind: FieldKind::Text { default: "" },
        required: false,
        visible_when: None,
        hint: Some("CSS border, e.g. 1px solid #000"),
        placeholder: Some("e.g. 1px solid #000000"),
        brand_placeholder: None,
    }
}

const fn border_sides_field() -> FormField {
    FormField {
        id: "border_sides",
        label: "Border sides",
        kind: FieldKind::Checkboxes {
            options: crate::border::OPTIONS,
            default: "all",
        },
        required: false,
        visible_when: None,
        hint: Some("Space: toggle  ·  ←/→: move"),
        placeholder: None,
        brand_placeholder: None,
    }
}

const fn inner_border_sides_field() -> FormField {
    FormField {
        id: "inner_border_sides",
        label: "Inner border sides",
        kind: FieldKind::Checkboxes {
            options: crate::border::OPTIONS,
            default: "all",
        },
        required: false,
        visible_when: None,
        hint: Some("Space: toggle  ·  ←/→: move"),
        placeholder: None,
        brand_placeholder: None,
    }
}

const fn border_radius_field() -> FormField {
    hinted(
        "border_radius",
        "Border radius",
        crate::padding::UNIT_HINT,
        crate::padding::UNIT_PLACEHOLDER,
    )
}

const fn author_label_field() -> FormField {
    FormField {
        id: "label",
        label: "Label",
        kind: FieldKind::Text { default: "" },
        required: false,
        visible_when: None,
        hint: Some("Tree and blueprint only — not in the email"),
        placeholder: Some("e.g. hero, features, footer"),
        brand_placeholder: None,
    }
}

const fn css_class_field() -> FormField {
    f(
        "css_class",
        "CSS class",
        FieldKind::Text { default: "" },
        false,
    )
}

const fn hamburger_only(id: &'static str, label: &'static str, kind: FieldKind) -> FormField {
    FormField {
        id,
        label,
        kind,
        required: false,
        visible_when: Some(FieldPredicate::FieldEquals {
            other_id: "hamburger",
            value: "true",
        }),
        hint: None,
        placeholder: None,
        brand_placeholder: None,
    }
}

pub const ALIGN_OPTIONS: &[&str] = &["", "left", "center", "right"];
pub const BOOL_OPTIONS: &[&str] = &["false", "true"];
pub const FONT_WEIGHT_OPTIONS: &[&str] = &["", "normal", "bold", "400", "700"];
pub const FONT_STYLE_OPTIONS: &[&str] = &["", "normal", "italic"];
pub const BORDER_STYLE_OPTIONS: &[&str] = &["", "solid", "dashed", "dotted", "none"];
pub const TEXT_DECORATION_OPTIONS: &[&str] = &["", "none", "underline", "overline", "line-through"];
pub const TEXT_TRANSFORM_OPTIONS: &[&str] = &["", "none", "uppercase", "lowercase", "capitalize"];
pub const HERO_MODE_OPTIONS: &[&str] = &["fluid-height", "fixed-height"];
pub const SOCIAL_MODE_OPTIONS: &[&str] = &["horizontal", "vertical"];
pub const IMAGE_POS_OPTIONS: &[&str] = &["top", "left", "right"];
pub const SOCIAL_NET_OPTIONS: &[&str] = &[
    "facebook",
    "instagram",
    "linkedin",
    "x",
    "github",
    "youtube",
    "pinterest",
    "google",
    "tumblr",
    "snapchat",
    "vimeo",
    "medium",
    "soundcloud",
    "dribbble",
    "xing",
    "web",
];
pub const ICON_POSITION_OPTIONS: &[&str] = &["", "left", "right"];
pub const TABLE_ROLE_OPTIONS: &[&str] = &["", "none", "presentation"];
pub const THUMBNAILS_OPTIONS: &[&str] = &["hidden", "visible", "supported"];
pub const DIR_OPTIONS: &[&str] = &["", "auto", "ltr", "rtl"];
pub const DIRECTION_OPTIONS: &[&str] = &["", "ltr", "rtl"];
pub const VERTICAL_ALIGN_OPTIONS: &[&str] = &["", "top", "middle", "bottom"];
pub const BG_REPEAT_OPTIONS: &[&str] = &["", "no-repeat", "repeat"];
pub const TARGET_OPTIONS: &[&str] = &["", "_blank", "_self"];

const fn direction_field() -> FormField {
    f(
        "direction",
        "Direction",
        FieldKind::Enum {
            options: DIRECTION_OPTIONS,
            default: "",
        },
        false,
    )
}

const fn vertical_align_field() -> FormField {
    f(
        "vertical_align",
        "Vertical align",
        FieldKind::Enum {
            options: VERTICAL_ALIGN_OPTIONS,
            default: "",
        },
        false,
    )
}

const fn background_url_field() -> FormField {
    f(
        "background_url",
        "Background URL",
        FieldKind::Url { default: "" },
        false,
    )
}

const fn background_size_field() -> FormField {
    FormField {
        id: "background_size",
        label: "Background size",
        kind: FieldKind::Text { default: "" },
        required: false,
        visible_when: None,
        hint: Some("auto, cover, contain, or px/%"),
        placeholder: Some("e.g. cover"),
        brand_placeholder: None,
    }
}

const fn background_repeat_field() -> FormField {
    f(
        "background_repeat",
        "Background repeat",
        FieldKind::Enum {
            options: BG_REPEAT_OPTIONS,
            default: "",
        },
        false,
    )
}

pub static FONT_ITEM_FORM: EditForm = EditForm {
    title: "font",
    fields: &[
        f("name", "Name", FieldKind::Text { default: "Raleway" }, true),
        f(
            "href",
            "Google Fonts href",
            FieldKind::Url {
                default: "https://fonts.googleapis.com/css2?family=Raleway:wght@400;700&display=swap",
            },
            true,
        ),
    ],
};

pub static SOCIAL_ITEM_FORM: EditForm = EditForm {
    title: "social icon",
    fields: &[
        f(
            "name",
            "Network",
            FieldKind::Enum {
                options: SOCIAL_NET_OPTIONS,
                default: "x",
            },
            true,
        ),
        f(
            "href",
            "Href",
            FieldKind::Url {
                default: "https://example.com",
            },
            true,
        ),
        f(
            "src",
            "Icon src (web)",
            FieldKind::Url { default: "" },
            false,
        ),
        f("alt", "Alt", FieldKind::Text { default: "" }, false),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        hinted(
            "icon_size",
            "Icon size",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        padding_field(),
        css_class_field(),
    ],
};

pub static HEAD_FORM: EditForm = EditForm {
    title: "mj-head",
    fields: &[
        f("subject", "Subject", FieldKind::Text { default: "" }, true),
        f(
            "preheader",
            "Preheader",
            FieldKind::Text { default: "" },
            false,
        ),
        f("lang", "Lang", FieldKind::Text { default: "en" }, true),
        f(
            "dir",
            "Dir",
            FieldKind::Enum {
                options: DIR_OPTIONS,
                default: "",
            },
            false,
        ),
        f("title", "Title", FieldKind::Text { default: "" }, true),
        f(
            "breakpoint",
            "Breakpoint",
            FieldKind::Text { default: "480px" },
            false,
        ),
        f(
            "base_url",
            "Base URL",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "fonts",
            "Fonts",
            FieldKind::SubForm {
                template: &FONT_ITEM_FORM,
                min_items: 0,
                summary_field_id: "name",
            },
            false,
        ),
        f(
            "json_ld",
            "JSON-LD",
            FieldKind::Textarea {
                rows: 6,
                default: "",
            },
            false,
        ),
        f(
            "css",
            "Custom CSS",
            FieldKind::Textarea {
                rows: 6,
                default: "",
            },
            false,
        ),
        f(
            "css_inline",
            "Inline CSS",
            FieldKind::Enum {
                options: BOOL_OPTIONS,
                default: "false",
            },
            false,
        ),
    ],
};

pub static BRAND_FORM: EditForm = EditForm {
    title: "brand",
    fields: &[
        f(
            "font_family",
            "Font family",
            FieldKind::Text { default: "" },
            true,
        ),
        f(
            "text_color",
            "Text color",
            FieldKind::Text { default: "#1a1a1a" },
            true,
        ),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "#f4f4f4" },
            true,
        ),
        f(
            "content_width",
            "Content width",
            FieldKind::Text { default: "600" },
            true,
        ),
        f(
            "button_background",
            "Button background",
            FieldKind::Text { default: "#FFAF46" },
            true,
        ),
        f(
            "button_color",
            "Button color",
            FieldKind::Text { default: "#0F1114" },
            true,
        ),
    ],
};

pub static BODY_FORM: EditForm = EditForm {
    title: "mj-body",
    fields: &[body_bg_field(), css_class_field()],
};

pub static SECTION_FORM: EditForm = EditForm {
    title: "mj-section",
    fields: &[
        author_label_field(),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        background_url_field(),
        background_size_field(),
        background_repeat_field(),
        padding_field(),
        hinted("gutter", "Gutter", crate::padding::UNIT_HINT, "e.g. 4%"),
        direction_field(),
        border_field(),
        border_sides_field(),
        border_radius_field(),
        f(
            "full_width",
            "Full width",
            FieldKind::Enum {
                options: BOOL_OPTIONS,
                default: "false",
            },
            false,
        ),
        css_class_field(),
    ],
};

pub static COLUMN_FORM: EditForm = EditForm {
    title: "mj-column",
    fields: &[
        f("width", "Width", FieldKind::Text { default: "" }, false),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        padding_field(),
        f(
            "inner_background_color",
            "Inner background",
            FieldKind::Text { default: "" },
            false,
        ),
        border_field(),
        border_sides_field(),
        border_radius_field(),
        f(
            "inner_border",
            "Inner border",
            FieldKind::Text { default: "" },
            false,
        ),
        inner_border_sides_field(),
        hinted(
            "inner_border_radius",
            "Inner border radius",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        vertical_align_field(),
        css_class_field(),
    ],
};

pub static WRAPPER_FORM: EditForm = EditForm {
    title: "mj-wrapper",
    fields: &[
        author_label_field(),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        background_url_field(),
        background_size_field(),
        background_repeat_field(),
        padding_field(),
        hinted(
            "gap",
            "Gap",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        border_field(),
        border_sides_field(),
        border_radius_field(),
        f(
            "full_width",
            "Full width",
            FieldKind::Enum {
                options: BOOL_OPTIONS,
                default: "false",
            },
            false,
        ),
        css_class_field(),
    ],
};

pub static GROUP_FORM: EditForm = EditForm {
    title: "mj-group",
    fields: &[
        f("width", "Width", FieldKind::Text { default: "" }, false),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        direction_field(),
        vertical_align_field(),
        css_class_field(),
    ],
};

pub static HERO_FORM: EditForm = EditForm {
    title: "mj-hero",
    fields: &[
        f(
            "mode",
            "Mode",
            FieldKind::Enum {
                options: HERO_MODE_OPTIONS,
                default: "fluid-height",
            },
            true,
        ),
        f(
            "background_url",
            "Background URL",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "background_height",
            "Background height",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "background_width",
            "Background width",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "background_position",
            "Background position",
            FieldKind::Text { default: "" },
            false,
        ),
        f("width", "Width", FieldKind::Text { default: "" }, false),
        f("height", "Height", FieldKind::Text { default: "" }, false),
        padding_field(),
        inner_padding_field(),
        border_radius_field(),
        vertical_align_field(),
        css_class_field(),
    ],
};

pub static TEXT_FORM: EditForm = EditForm {
    title: "mj-text",
    fields: &[
        f(
            "content",
            "Content",
            FieldKind::Textarea {
                rows: 6,
                default: "",
            },
            true,
        ),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "font_size",
            "Font size",
            FieldKind::Text { default: "" },
            false,
        ),
        font_family_field(),
        f(
            "font_weight",
            "Font weight",
            FieldKind::Enum {
                options: FONT_WEIGHT_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "font_style",
            "Font style",
            FieldKind::Enum {
                options: FONT_STYLE_OPTIONS,
                default: "",
            },
            false,
        ),
        hinted(
            "line_height",
            "Line height",
            crate::padding::UNIT_HINT,
            "e.g. 1.5  or  24px",
        ),
        text_color_field(),
        padding_field(),
        hinted(
            "letter_spacing",
            "Letter spacing",
            crate::padding::UNIT_HINT,
            "e.g. 0.5px",
        ),
        f(
            "text_decoration",
            "Text decoration",
            FieldKind::Enum {
                options: TEXT_DECORATION_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "text_transform",
            "Text transform",
            FieldKind::Enum {
                options: TEXT_TRANSFORM_OPTIONS,
                default: "",
            },
            false,
        ),
        hinted("height", "Height", crate::padding::UNIT_HINT, "e.g. 24px"),
        css_class_field(),
    ],
};

pub static BUTTON_FORM: EditForm = EditForm {
    title: "mj-button",
    fields: &[
        f(
            "content",
            "Label",
            FieldKind::Text {
                default: "Read more",
            },
            true,
        ),
        f(
            "href",
            "Href",
            FieldKind::Url {
                default: "https://example.com",
            },
            true,
        ),
        button_bg_field(),
        button_color_field(),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        font_family_field(),
        hinted(
            "font_size",
            "Font size",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        f(
            "font_weight",
            "Font weight",
            FieldKind::Enum {
                options: FONT_WEIGHT_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "font_style",
            "Font style",
            FieldKind::Enum {
                options: FONT_STYLE_OPTIONS,
                default: "",
            },
            false,
        ),
        border_field(),
        border_sides_field(),
        border_radius_field(),
        inner_padding_field(),
        f("width", "Width", FieldKind::Text { default: "" }, false),
        hinted("height", "Height", crate::padding::UNIT_HINT, "e.g. 44px"),
        f(
            "target",
            "Target",
            FieldKind::Enum {
                options: TARGET_OPTIONS,
                default: "",
            },
            false,
        ),
        padding_field(),
        hinted(
            "letter_spacing",
            "Letter spacing",
            crate::padding::UNIT_HINT,
            "e.g. 0.5px",
        ),
        hinted(
            "line_height",
            "Line height",
            crate::padding::UNIT_HINT,
            "e.g. 120%",
        ),
        f(
            "text_decoration",
            "Text decoration",
            FieldKind::Enum {
                options: TEXT_DECORATION_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "text_transform",
            "Text transform",
            FieldKind::Enum {
                options: TEXT_TRANSFORM_OPTIONS,
                default: "",
            },
            false,
        ),
        f("rel", "Rel", FieldKind::Text { default: "" }, false),
        f("title", "Title", FieldKind::Text { default: "" }, false),
        css_class_field(),
    ],
};

pub static IMAGE_FORM: EditForm = EditForm {
    title: "mj-image",
    fields: &[
        f("src", "Src", FieldKind::Url { default: "" }, true),
        f("alt", "Alt", FieldKind::Text { default: "" }, true),
        f("href", "Href", FieldKind::Url { default: "" }, false),
        f("title", "Title", FieldKind::Text { default: "" }, false),
        f("width", "Width", FieldKind::Text { default: "" }, false),
        hinted(
            "height",
            "Height",
            crate::padding::UNIT_HINT,
            "e.g. 200px  or  auto",
        ),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "fluid_on_mobile",
            "Fluid on mobile",
            FieldKind::Enum {
                options: BOOL_OPTIONS,
                default: "true",
            },
            false,
        ),
        border_field(),
        border_sides_field(),
        border_radius_field(),
        padding_field(),
        f(
            "target",
            "Target",
            FieldKind::Enum {
                options: TARGET_OPTIONS,
                default: "",
            },
            false,
        ),
        f("rel", "Rel", FieldKind::Text { default: "" }, false),
        css_class_field(),
    ],
};

pub static DIVIDER_FORM: EditForm = EditForm {
    title: "mj-divider",
    fields: &[
        f(
            "border_color",
            "Border color",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "border_width",
            "Border width",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "border_style",
            "Border style",
            FieldKind::Enum {
                options: BORDER_STYLE_OPTIONS,
                default: "",
            },
            false,
        ),
        f("width", "Width", FieldKind::Text { default: "" }, false),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        padding_field(),
        css_class_field(),
    ],
};

pub static SPACER_FORM: EditForm = EditForm {
    title: "mj-spacer",
    fields: &[
        f(
            "height",
            "Height",
            FieldKind::Text { default: "24px" },
            true,
        ),
        padding_field(),
        css_class_field(),
    ],
};

pub static SOCIAL_FORM: EditForm = EditForm {
    title: "mj-social",
    fields: &[
        f(
            "mode",
            "Mode",
            FieldKind::Enum {
                options: SOCIAL_MODE_OPTIONS,
                default: "horizontal",
            },
            true,
        ),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "icon_size",
            "Icon size",
            FieldKind::Text { default: "32px" },
            false,
        ),
        border_radius_field(),
        padding_field(),
        hinted(
            "icon_padding",
            "Icon padding",
            crate::padding::HINT,
            crate::padding::PLACEHOLDER,
        ),
        inner_padding_field(),
        hinted(
            "font_size",
            "Font size",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        f("color", "Color", FieldKind::Text { default: "" }, false),
        f(
            "elements",
            "Icons",
            FieldKind::SubForm {
                template: &SOCIAL_ITEM_FORM,
                min_items: 0,
                summary_field_id: "name",
            },
            false,
        ),
        css_class_field(),
    ],
};

pub static NAVBAR_FORM: EditForm = EditForm {
    title: "mj-navbar",
    fields: &[
        f(
            "hamburger",
            "Hamburger",
            FieldKind::Enum {
                options: BOOL_OPTIONS,
                default: "false",
            },
            false,
        ),
        f(
            "ico_color",
            "Icon color",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "base_url",
            "Base URL",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        padding_field(),
        hamburger_only(
            "ico_align",
            "Icon align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
        ),
        hamburger_only(
            "ico_font_size",
            "Icon font size",
            FieldKind::Text { default: "" },
        ),
        hamburger_only(
            "ico_padding",
            "Icon padding",
            FieldKind::Text { default: "" },
        ),
        hamburger_only("ico_open", "Icon open", FieldKind::Text { default: "" }),
        hamburger_only("ico_close", "Icon close", FieldKind::Text { default: "" }),
        css_class_field(),
    ],
};

pub static NAVBAR_LINK_FORM: EditForm = EditForm {
    title: "mj-navbar-link",
    fields: &[
        f(
            "href",
            "Href",
            FieldKind::Url {
                default: "https://example.com",
            },
            true,
        ),
        f(
            "content",
            "Label",
            FieldKind::Text { default: "Link" },
            true,
        ),
        f("color", "Color", FieldKind::Text { default: "" }, false),
        font_family_field(),
        hinted(
            "font_size",
            "Font size",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        f(
            "font_weight",
            "Font weight",
            FieldKind::Enum {
                options: FONT_WEIGHT_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "text_decoration",
            "Text decoration",
            FieldKind::Enum {
                options: TEXT_DECORATION_OPTIONS,
                default: "",
            },
            false,
        ),
        f(
            "text_transform",
            "Text transform",
            FieldKind::Enum {
                options: TEXT_TRANSFORM_OPTIONS,
                default: "",
            },
            false,
        ),
        padding_field(),
        css_class_field(),
    ],
};

pub static ACCORDION_FORM: EditForm = EditForm {
    title: "mj-accordion",
    fields: &[
        f("border", "Border", FieldKind::Text { default: "" }, false),
        padding_field(),
        font_family_field(),
        f(
            "icon_position",
            "Icon position",
            FieldKind::Enum {
                options: ICON_POSITION_OPTIONS,
                default: "",
            },
            false,
        ),
        hinted(
            "icon_width",
            "Icon width",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        hinted(
            "icon_height",
            "Icon height",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        f(
            "icon_wrapped_url",
            "Icon wrapped URL",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "icon_unwrapped_url",
            "Icon unwrapped URL",
            FieldKind::Url { default: "" },
            false,
        ),
        css_class_field(),
    ],
};

pub static ACCORDION_ELEMENT_FORM: EditForm = EditForm {
    title: "mj-accordion-element",
    fields: &[
        f("title", "Title", FieldKind::Text { default: "Title" }, true),
        f(
            "content",
            "Content",
            FieldKind::Textarea {
                rows: 6,
                default: "",
            },
            false,
        ),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
        css_class_field(),
    ],
};

pub static CAROUSEL_FORM: EditForm = EditForm {
    title: "mj-carousel",
    fields: &[
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        padding_field(),
        border_radius_field(),
        hinted(
            "tb_border_radius",
            "Thumbnail radius",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        hinted(
            "icon_width",
            "Icon width",
            crate::padding::UNIT_HINT,
            crate::padding::UNIT_PLACEHOLDER,
        ),
        f(
            "thumbnails",
            "Thumbnails",
            FieldKind::Enum {
                options: THUMBNAILS_OPTIONS,
                default: "hidden",
            },
            true,
        ),
        css_class_field(),
    ],
};

pub static CAROUSEL_IMAGE_FORM: EditForm = EditForm {
    title: "mj-carousel-image",
    fields: &[
        f("src", "Src", FieldKind::Url { default: "" }, true),
        f("alt", "Alt", FieldKind::Text { default: "" }, true),
        f("href", "Href", FieldKind::Url { default: "" }, false),
        f(
            "thumbnails_src",
            "Thumbnail src",
            FieldKind::Url { default: "" },
            false,
        ),
        border_radius_field(),
        css_class_field(),
    ],
};

pub static TABLE_FORM: EditForm = EditForm {
    title: "mj-table",
    fields: &[
        FormField {
            id: "content",
            label: "Table HTML",
            kind: FieldKind::Textarea {
                rows: 8,
                default: "",
            },
            required: true,
            visible_when: None,
            hint: Some("Rows only: <tr><td>…</td></tr>. mj-table already emits <table>."),
            placeholder: Some("<tr><td></td></tr>"),
            brand_placeholder: None,
        },
        f(
            "font_size",
            "Font size",
            FieldKind::Text { default: "" },
            false,
        ),
        font_family_field(),
        hinted(
            "line_height",
            "Line height",
            crate::padding::UNIT_HINT,
            "e.g. 22px",
        ),
        f("color", "Color", FieldKind::Text { default: "" }, false),
        f(
            "align",
            "Align",
            FieldKind::Enum {
                options: ALIGN_OPTIONS,
                default: "",
            },
            false,
        ),
        f("width", "Width", FieldKind::Text { default: "" }, false),
        border_field(),
        padding_field(),
        f(
            "cellpadding",
            "Cell padding",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "cellspacing",
            "Cell spacing",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "role",
            "Role",
            FieldKind::Enum {
                options: TABLE_ROLE_OPTIONS,
                default: "",
            },
            false,
        ),
        css_class_field(),
    ],
};

pub static EMAIL_HEADER_FORM: EditForm = EditForm {
    title: "email-header",
    fields: &[
        f(
            "logo_src",
            "Logo src",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "logo_alt",
            "Logo alt",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "logo_href",
            "Logo href",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "logo_width",
            "Logo width",
            FieldKind::Text { default: "160px" },
            false,
        ),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
    ],
};

pub static EMAIL_HERO_FORM: EditForm = EditForm {
    title: "email-hero",
    fields: &[
        f(
            "image_src",
            "Image src",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "image_alt",
            "Image alt",
            FieldKind::Text { default: "" },
            false,
        ),
        f("heading", "Heading", FieldKind::Text { default: "" }, false),
        f(
            "subheading",
            "Subheading",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
    ],
};

pub static EMAIL_CTA_FORM: EditForm = EditForm {
    title: "email-cta",
    fields: &[
        f("heading", "Heading", FieldKind::Text { default: "" }, false),
        f(
            "copy",
            "Copy",
            FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            false,
        ),
        f(
            "button_label",
            "Button label",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "button_href",
            "Button href",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "background_color",
            "Background",
            FieldKind::Text { default: "" },
            false,
        ),
    ],
};

pub static EMAIL_ARTICLE_FORM: EditForm = EditForm {
    title: "email-article",
    fields: &[
        f(
            "image_src",
            "Image src",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "image_alt",
            "Image alt",
            FieldKind::Text { default: "" },
            false,
        ),
        f("title", "Title", FieldKind::Text { default: "" }, false),
        f(
            "copy",
            "Copy",
            FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            false,
        ),
        f(
            "link_label",
            "Link label",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "link_href",
            "Link href",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "image_position",
            "Image position",
            FieldKind::Enum {
                options: IMAGE_POS_OPTIONS,
                default: "top",
            },
            false,
        ),
    ],
};

pub static EMAIL_FOOTER_FORM: EditForm = EditForm {
    title: "email-footer",
    fields: &[
        f(
            "company_name",
            "Company",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "address_lines",
            "Address (one line per row)",
            FieldKind::Textarea {
                rows: 4,
                default: "",
            },
            true,
        ),
        f(
            "unsubscribe_label",
            "Unsubscribe label",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "unsubscribe_href",
            "Unsubscribe href",
            FieldKind::Url { default: "" },
            false,
        ),
        f(
            "copyright",
            "Copyright",
            FieldKind::Text { default: "" },
            false,
        ),
        f(
            "social",
            "Social",
            FieldKind::SubForm {
                template: &SOCIAL_ITEM_FORM,
                min_items: 0,
                summary_field_id: "name",
            },
            false,
        ),
    ],
};
