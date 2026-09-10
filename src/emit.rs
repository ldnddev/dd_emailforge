//! Deterministic Template → self-contained MJML 5.
use crate::model::{
    Align, BodyNode, ColumnChild, EmailArticle, EmailCta, EmailFooter, EmailHeader, EmailHero,
    HeroMode, ImagePosition, MjAccordion, MjAccordionElement, MjButton, MjCarousel,
    MjCarouselImage, MjColumn, MjDivider, MjGroup, MjHero, MjImage, MjNavbar, MjNavbarLink,
    MjSection, MjSocial, MjSocialElement, MjSpacer, MjTable, MjText, MjWrapper, SectionChild,
    SocialMode, SocialNetwork, Template, Thumbnails,
};

#[derive(Debug, Clone)]
pub enum EmitMode {
    Preview { origin: String },
    Export,
}

/// Written on `<mj-button>` in `mj-attributes`. MJML subtracts this from a
/// pixel `width` when sizing the inner `<a>`, so keep it in sync with validate.
pub const BRAND_BUTTON_INNER_PADDING: &str = "12px 24px";

pub fn emit_mjml(t: &Template, mode: EmitMode) -> anyhow::Result<String> {
    let mut w = Writer::new();
    let mut mjml_attrs = vec![("lang", xml_escape_attr(&t.lang))];
    if !t.dir.trim().is_empty() {
        mjml_attrs.push(("dir", xml_escape_attr(&t.dir)));
    }
    w.open_attrs("mjml", &mjml_attrs);
    w.open("mj-head");
    w.leaf_text("mj-title", &xml_escape_text(&t.head.title));
    if !t.preheader.trim().is_empty() {
        w.leaf_text("mj-preview", &xml_escape_text(&t.preheader));
    }
    w.self_close(
        "mj-breakpoint",
        &[("width", xml_escape_attr(&t.head.breakpoint))],
    );
    for font in &t.head.fonts {
        w.self_close(
            "mj-font",
            &[
                ("name", xml_escape_attr(&font.name)),
                ("href", xml_escape_attr(&font.href)),
            ],
        );
    }
    emit_attributes(&mut w, t);
    w.open("mj-style");
    w.raw_indented(
        ".preheader { display:none !important; visibility:hidden; opacity:0; color:transparent; height:0; width:0; }",
    );
    w.close("mj-style");
    if !t.head.css.trim().is_empty() {
        if t.head.css_inline {
            w.open_attrs("mj-style", &[("inline", "inline".to_string())]);
        } else {
            w.open("mj-style");
        }
        w.raw_indented(t.head.css.trim());
        w.close("mj-style");
    }
    let json_ld = emit_json_ld_payload(&t.head.json_ld)?;
    if let Some(ref payload) = json_ld {
        emit_json_ld_raw(&mut w, payload);
    }
    w.close("mj-head");

    let body_bg = if t.body.background_color.trim().is_empty() {
        t.brand.background_color.as_str()
    } else {
        t.body.background_color.as_str()
    };
    let mut body_attrs = vec![
        ("background-color", xml_escape_attr(body_bg)),
        ("width", format!("{}px", t.brand.content_width)),
    ];
    push_opt(&mut body_attrs, "css-class", t.body.css_class.as_deref());
    w.open_attrs("mj-body", &body_attrs);
    if !t.preheader.trim().is_empty() {
        w.open_attrs(
            "mj-section",
            &[("css-class", "preheader".into()), ("padding", "0".into())],
        );
        w.open("mj-column");
        w.leaf_text("mj-text", &xml_escape_text(&t.preheader));
        w.close("mj-column");
        w.close("mj-section");
    }
    for node in &t.body.nodes {
        emit_body_node(&mut w, t, &mode, node)?;
    }
    w.close("mj-body");
    w.close("mjml");
    Ok(w.finish())
}

pub fn write_mjml(t: &Template, path: &std::path::Path, mode: EmitMode) -> anyhow::Result<()> {
    let mjml = emit_mjml(t, mode)?;
    crate::storage::atomic_write(path, mjml.as_bytes())
}

fn emit_attributes(w: &mut Writer, t: &Template) {
    w.open("mj-attributes");
    w.self_close(
        "mj-all",
        &[("font-family", xml_escape_attr(&t.brand.font_family))],
    );
    w.self_close(
        "mj-text",
        &[
            ("color", xml_escape_attr(&t.brand.text_color)),
            ("font-size", "16px".into()),
            ("line-height", "1.5".into()),
        ],
    );
    w.self_close(
        "mj-button",
        &[
            (
                "background-color",
                xml_escape_attr(&t.brand.button_background),
            ),
            ("color", xml_escape_attr(&t.brand.button_color)),
            ("border-radius", "4px".into()),
            ("inner-padding", BRAND_BUTTON_INNER_PADDING.into()),
            ("font-weight", "bold".into()),
        ],
    );
    w.self_close(
        "mj-body",
        &[
            (
                "background-color",
                xml_escape_attr(if t.body.background_color.trim().is_empty() {
                    &t.brand.background_color
                } else {
                    &t.body.background_color
                }),
            ),
            ("width", format!("{}px", t.brand.content_width)),
        ],
    );
    w.self_close("mj-section", &[("padding", "20px 0".into())]);
    w.self_close("mj-image", &[("padding", "0".into())]);
    w.close("mj-attributes");
}

fn emit_json_ld_payload(raw: &str) -> anyhow::Result<Option<String>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| anyhow::anyhow!("head.json_ld is not valid JSON: {e}"))?;
    match value {
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
            Ok(Some(serde_json::to_string_pretty(&value)?))
        }
        _ => anyhow::bail!("head.json_ld must be a JSON object or array"),
    }
}

fn emit_json_ld_raw(w: &mut Writer, payload: &str) {
    // Preferred placement: inside mj-head (Key Decision 18).
    w.open("mj-raw");
    w.raw_indented("<script type=\"application/ld+json\">");
    for line in payload.lines() {
        w.raw_indented(line);
    }
    w.raw_indented("</script>");
    w.close("mj-raw");
}

fn emit_body_node(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    node: &BodyNode,
) -> anyhow::Result<()> {
    match node {
        BodyNode::MjSection(s) => emit_section(w, t, mode, s),
        BodyNode::MjWrapper(s) => emit_wrapper(w, t, mode, s),
        BodyNode::MjHero(s) => emit_hero(w, t, mode, s),
        BodyNode::EmailHeader(s) => emit_email_header(w, t, mode, s),
        BodyNode::EmailHero(s) => emit_email_hero(w, t, mode, s),
        BodyNode::EmailCta(s) => emit_email_cta(w, t, mode, s),
        BodyNode::EmailArticle(s) => emit_email_article(w, t, mode, s),
        BodyNode::EmailFooter(s) => emit_email_footer(w, t, mode, s),
    }
}

fn emit_section(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    s: &MjSection,
) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(
        &mut attrs,
        "background-color",
        s.background_color.as_deref(),
    );
    push_padding(&mut attrs, s.padding.as_deref());
    push_border(&mut attrs, s.border.as_deref(), s.border_sides.as_deref());
    push_unit(&mut attrs, "border-radius", s.border_radius.as_deref());
    push_unit(&mut attrs, "gutter", s.gutter.as_deref());
    push_bg_url(&mut attrs, s.background_url.as_deref(), t, mode)?;
    push_opt(&mut attrs, "background-size", s.background_size.as_deref());
    push_opt(
        &mut attrs,
        "background-repeat",
        s.background_repeat.as_deref(),
    );
    push_opt(&mut attrs, "direction", s.direction.as_deref());
    if s.full_width {
        attrs.push(("full-width", "full-width".into()));
    }
    push_opt(&mut attrs, "css-class", s.css_class.as_deref());
    w.open_attrs("mj-section", &attrs);
    for child in &s.children {
        match child {
            SectionChild::MjColumn(c) => emit_column(w, t, mode, c)?,
            SectionChild::MjGroup(g) => emit_group(w, t, mode, g)?,
        }
    }
    w.close("mj-section");
    Ok(())
}

fn emit_wrapper(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    s: &MjWrapper,
) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(
        &mut attrs,
        "background-color",
        s.background_color.as_deref(),
    );
    push_padding(&mut attrs, s.padding.as_deref());
    push_border(&mut attrs, s.border.as_deref(), s.border_sides.as_deref());
    push_unit(&mut attrs, "border-radius", s.border_radius.as_deref());
    push_bg_url(&mut attrs, s.background_url.as_deref(), t, mode)?;
    push_opt(&mut attrs, "background-size", s.background_size.as_deref());
    push_opt(
        &mut attrs,
        "background-repeat",
        s.background_repeat.as_deref(),
    );
    push_unit(&mut attrs, "gap", s.gap.as_deref());
    if s.full_width {
        attrs.push(("full-width", "full-width".into()));
    }
    push_opt(&mut attrs, "css-class", s.css_class.as_deref());
    w.open_attrs("mj-wrapper", &attrs);
    for child in &s.children {
        emit_body_node(w, t, mode, child)?;
    }
    w.close("mj-wrapper");
    Ok(())
}

fn emit_group(w: &mut Writer, t: &Template, mode: &EmitMode, g: &MjGroup) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "width", g.width.as_deref());
    push_opt(
        &mut attrs,
        "background-color",
        g.background_color.as_deref(),
    );
    push_opt(&mut attrs, "direction", g.direction.as_deref());
    push_opt(&mut attrs, "vertical-align", g.vertical_align.as_deref());
    push_opt(&mut attrs, "css-class", g.css_class.as_deref());
    w.open_attrs("mj-group", &attrs);
    for col in &g.children {
        emit_column(w, t, mode, col)?;
    }
    w.close("mj-group");
    Ok(())
}

fn emit_column(w: &mut Writer, t: &Template, mode: &EmitMode, c: &MjColumn) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "width", c.width.as_deref());
    push_opt(
        &mut attrs,
        "background-color",
        c.background_color.as_deref(),
    );
    push_padding(&mut attrs, c.padding.as_deref());
    push_opt(
        &mut attrs,
        "inner-background-color",
        c.inner_background_color.as_deref(),
    );
    push_border(&mut attrs, c.border.as_deref(), c.border_sides.as_deref());
    push_unit(&mut attrs, "border-radius", c.border_radius.as_deref());
    push_border_named(
        &mut attrs,
        "inner-border",
        c.inner_border.as_deref(),
        c.inner_border_sides.as_deref(),
    );
    push_unit(
        &mut attrs,
        "inner-border-radius",
        c.inner_border_radius.as_deref(),
    );
    push_opt(&mut attrs, "vertical-align", c.vertical_align.as_deref());
    push_opt(&mut attrs, "css-class", c.css_class.as_deref());
    w.open_attrs("mj-column", &attrs);
    for child in &c.components {
        emit_column_child(w, t, mode, child)?;
    }
    w.close("mj-column");
    Ok(())
}

fn emit_hero(w: &mut Writer, t: &Template, mode: &EmitMode, h: &MjHero) -> anyhow::Result<()> {
    let mut attrs = vec![(
        "mode",
        match h.mode {
            HeroMode::FluidHeight => "fluid-height",
            HeroMode::FixedHeight => "fixed-height",
        }
        .to_string(),
    )];
    if let Some(url) = h
        .background_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        attrs.push((
            "background-url",
            xml_escape_attr(&rewrite_src(url, t, mode)?),
        ));
    }
    push_opt(
        &mut attrs,
        "background-color",
        h.background_color.as_deref(),
    );
    push_opt(
        &mut attrs,
        "background-height",
        h.background_height.as_deref(),
    );
    push_opt(
        &mut attrs,
        "background-width",
        h.background_width.as_deref(),
    );
    push_opt(
        &mut attrs,
        "background-position",
        h.background_position.as_deref(),
    );
    push_opt(&mut attrs, "width", h.width.as_deref());
    push_opt(&mut attrs, "height", h.height.as_deref());
    push_padding(&mut attrs, h.padding.as_deref());
    push_padding_named(&mut attrs, "inner-padding", h.inner_padding.as_deref());
    push_unit(&mut attrs, "border-radius", h.border_radius.as_deref());
    push_opt(&mut attrs, "vertical-align", h.vertical_align.as_deref());
    push_opt(&mut attrs, "css-class", h.css_class.as_deref());
    w.open_attrs("mj-hero", &attrs);
    for child in &h.children {
        emit_column_child(w, t, mode, child)?;
    }
    w.close("mj-hero");
    Ok(())
}

fn emit_column_child(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    child: &ColumnChild,
) -> anyhow::Result<()> {
    match child {
        ColumnChild::MjText(n) => emit_text(w, n),
        ColumnChild::MjButton(n) => emit_button(w, n),
        ColumnChild::MjImage(n) => emit_image(w, t, mode, n),
        ColumnChild::MjDivider(n) => emit_divider(w, n),
        ColumnChild::MjSpacer(n) => emit_spacer(w, n),
        ColumnChild::MjSocial(n) => emit_social(w, n),
        ColumnChild::MjTable(n) => emit_table(w, n),
        ColumnChild::MjNavbar(n) => emit_navbar(w, n),
        ColumnChild::MjAccordion(n) => emit_accordion(w, n),
        ColumnChild::MjCarousel(n) => emit_carousel(w, t, mode, n),
    }
}

fn emit_text(w: &mut Writer, n: &MjText) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_align(&mut attrs, n.align);
    push_opt(&mut attrs, "font-size", n.font_size.as_deref());
    push_opt(&mut attrs, "font-family", n.font_family.as_deref());
    push_opt(&mut attrs, "font-weight", n.font_weight.as_deref());
    push_opt(&mut attrs, "font-style", n.font_style.as_deref());
    push_opt(&mut attrs, "line-height", n.line_height.as_deref());
    push_opt(&mut attrs, "color", n.color.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "letter-spacing", n.letter_spacing.as_deref());
    push_opt(&mut attrs, "text-decoration", n.text_decoration.as_deref());
    push_opt(&mut attrs, "text-transform", n.text_transform.as_deref());
    push_opt(&mut attrs, "height", n.height.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-text", &attrs);
    w.raw_indented(&emit_text_inner(&n.content));
    w.close("mj-text");
    Ok(())
}

fn emit_button(w: &mut Writer, n: &MjButton) -> anyhow::Result<()> {
    let mut attrs = vec![("href", xml_escape_attr(&n.href))];
    push_opt(
        &mut attrs,
        "background-color",
        n.background_color.as_deref(),
    );
    push_opt(&mut attrs, "color", n.color.as_deref());
    push_align(&mut attrs, n.align);
    push_opt(&mut attrs, "font-family", n.font_family.as_deref());
    push_opt(&mut attrs, "font-size", n.font_size.as_deref());
    push_opt(&mut attrs, "font-weight", n.font_weight.as_deref());
    push_opt(&mut attrs, "font-style", n.font_style.as_deref());
    push_border(&mut attrs, n.border.as_deref(), n.border_sides.as_deref());
    push_unit(&mut attrs, "border-radius", n.border_radius.as_deref());
    push_padding_named(&mut attrs, "inner-padding", n.inner_padding.as_deref());
    push_opt(&mut attrs, "width", n.width.as_deref());
    push_opt(&mut attrs, "height", n.height.as_deref());
    push_opt(&mut attrs, "target", n.target.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "letter-spacing", n.letter_spacing.as_deref());
    push_opt(&mut attrs, "line-height", n.line_height.as_deref());
    push_opt(&mut attrs, "text-decoration", n.text_decoration.as_deref());
    push_opt(&mut attrs, "text-transform", n.text_transform.as_deref());
    push_opt(&mut attrs, "rel", n.rel.as_deref());
    push_opt(&mut attrs, "title", n.title.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-button", &attrs);
    w.raw_indented(&xml_escape_text(&n.content));
    w.close("mj-button");
    Ok(())
}

fn emit_image(w: &mut Writer, t: &Template, mode: &EmitMode, n: &MjImage) -> anyhow::Result<()> {
    let src = rewrite_src(&n.src, t, mode)?;
    let mut attrs = vec![
        ("src", xml_escape_attr(&src)),
        ("alt", xml_escape_attr(&n.alt)),
    ];
    if let Some(href) = n.href.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        attrs.push(("href", xml_escape_attr(href)));
    }
    push_opt(&mut attrs, "width", n.width.as_deref());
    push_opt(&mut attrs, "height", n.height.as_deref());
    push_align(&mut attrs, n.align);
    if n.fluid_on_mobile {
        attrs.push(("fluid-on-mobile", "true".into()));
    }
    push_border(&mut attrs, n.border.as_deref(), n.border_sides.as_deref());
    push_unit(&mut attrs, "border-radius", n.border_radius.as_deref());
    push_opt(&mut attrs, "title", n.title.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "target", n.target.as_deref());
    push_opt(&mut attrs, "rel", n.rel.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.self_close("mj-image", &attrs);
    Ok(())
}

fn emit_divider(w: &mut Writer, n: &MjDivider) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "border-color", n.border_color.as_deref());
    push_opt(&mut attrs, "border-width", n.border_width.as_deref());
    push_opt(&mut attrs, "border-style", n.border_style.as_deref());
    push_opt(&mut attrs, "width", n.width.as_deref());
    push_align(&mut attrs, n.align);
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.self_close("mj-divider", &attrs);
    Ok(())
}

fn emit_spacer(w: &mut Writer, n: &MjSpacer) -> anyhow::Result<()> {
    let mut attrs = vec![("height", xml_escape_attr(&n.height))];
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.self_close("mj-spacer", &attrs);
    Ok(())
}

fn emit_social(w: &mut Writer, n: &MjSocial) -> anyhow::Result<()> {
    let mut attrs = vec![(
        "mode",
        match n.mode {
            SocialMode::Horizontal => "horizontal",
            SocialMode::Vertical => "vertical",
        }
        .to_string(),
    )];
    push_align(&mut attrs, n.align);
    if !n.icon_size.trim().is_empty() {
        attrs.push(("icon-size", xml_escape_attr(&n.icon_size)));
    }
    push_unit(&mut attrs, "border-radius", n.border_radius.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_padding_named(&mut attrs, "icon-padding", n.icon_padding.as_deref());
    push_padding_named(&mut attrs, "inner-padding", n.inner_padding.as_deref());
    push_opt(&mut attrs, "font-size", n.font_size.as_deref());
    push_opt(&mut attrs, "color", n.color.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-social", &attrs);
    for el in &n.elements {
        emit_social_element(w, el);
    }
    w.close("mj-social");
    Ok(())
}

fn emit_social_element(w: &mut Writer, el: &MjSocialElement) {
    let mut attrs = vec![
        ("name", social_network_mjml(el.name).to_string()),
        ("href", xml_escape_attr(&el.href)),
    ];
    if let Some(src) = el.src.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        attrs.push(("src", xml_escape_attr(src)));
    }
    push_opt(
        &mut attrs,
        "background-color",
        el.background_color.as_deref(),
    );
    push_padding(&mut attrs, el.padding.as_deref());
    push_opt(&mut attrs, "icon-size", el.icon_size.as_deref());
    push_opt(&mut attrs, "alt", el.alt.as_deref());
    push_opt(&mut attrs, "css-class", el.css_class.as_deref());
    w.self_close("mj-social-element", &attrs);
}

fn social_network_mjml(n: SocialNetwork) -> &'static str {
    match n {
        SocialNetwork::Facebook => "facebook",
        SocialNetwork::Instagram => "instagram",
        SocialNetwork::Linkedin => "linkedin",
        // MJML 5.4 built-in is still `twitter` (Key Decision 20).
        SocialNetwork::X => "twitter",
        SocialNetwork::Github => "github",
        SocialNetwork::Web => "web",
        SocialNetwork::Youtube => "youtube",
        SocialNetwork::Pinterest => "pinterest",
        SocialNetwork::Google => "google",
        SocialNetwork::Tumblr => "tumblr",
        SocialNetwork::Snapchat => "snapchat",
        SocialNetwork::Vimeo => "vimeo",
        SocialNetwork::Medium => "medium",
        SocialNetwork::Soundcloud => "soundcloud",
        SocialNetwork::Dribbble => "dribbble",
        SocialNetwork::Xing => "xing",
    }
}

fn emit_navbar(w: &mut Writer, n: &MjNavbar) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    if n.hamburger {
        attrs.push(("hamburger", "hamburger".into()));
    }
    push_opt(&mut attrs, "ico-color", n.ico_color.as_deref());
    push_opt(&mut attrs, "base-url", n.base_url.as_deref());
    push_align(&mut attrs, n.align);
    push_padding(&mut attrs, n.padding.as_deref());
    if n.hamburger {
        push_opt(&mut attrs, "ico-align", n.ico_align.as_deref());
        push_opt(&mut attrs, "ico-font-size", n.ico_font_size.as_deref());
        push_padding_named(&mut attrs, "ico-padding", n.ico_padding.as_deref());
        push_opt(&mut attrs, "ico-open", n.ico_open.as_deref());
        push_opt(&mut attrs, "ico-close", n.ico_close.as_deref());
    }
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-navbar", &attrs);
    for link in &n.links {
        emit_navbar_link(w, link);
    }
    w.close("mj-navbar");
    Ok(())
}

fn emit_navbar_link(w: &mut Writer, n: &MjNavbarLink) {
    let mut attrs = vec![("href", xml_escape_attr(&n.href))];
    push_opt(&mut attrs, "color", n.color.as_deref());
    push_opt(&mut attrs, "font-family", n.font_family.as_deref());
    push_opt(&mut attrs, "font-size", n.font_size.as_deref());
    push_opt(&mut attrs, "font-weight", n.font_weight.as_deref());
    push_opt(&mut attrs, "text-decoration", n.text_decoration.as_deref());
    push_opt(&mut attrs, "text-transform", n.text_transform.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-navbar-link", &attrs);
    w.raw_indented(&xml_escape_text(&n.content));
    w.close("mj-navbar-link");
}

fn emit_accordion(w: &mut Writer, n: &MjAccordion) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "border", n.border.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "font-family", n.font_family.as_deref());
    push_opt(&mut attrs, "icon-position", n.icon_position.as_deref());
    push_opt(&mut attrs, "icon-width", n.icon_width.as_deref());
    push_opt(&mut attrs, "icon-height", n.icon_height.as_deref());
    push_opt(
        &mut attrs,
        "icon-wrapped-url",
        n.icon_wrapped_url.as_deref(),
    );
    push_opt(
        &mut attrs,
        "icon-unwrapped-url",
        n.icon_unwrapped_url.as_deref(),
    );
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-accordion", &attrs);
    for el in &n.elements {
        emit_accordion_element(w, el);
    }
    w.close("mj-accordion");
    Ok(())
}

fn emit_accordion_element(w: &mut Writer, n: &MjAccordionElement) {
    let mut attrs = Vec::new();
    push_opt(
        &mut attrs,
        "background-color",
        n.background_color.as_deref(),
    );
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-accordion-element", &attrs);
    w.open("mj-accordion-title");
    w.raw_indented(&xml_escape_text(&n.title));
    w.close("mj-accordion-title");
    w.open("mj-accordion-text");
    w.raw_indented(&emit_text_inner(&n.content));
    w.close("mj-accordion-text");
    w.close("mj-accordion-element");
}

fn emit_carousel(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    n: &MjCarousel,
) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_align(&mut attrs, n.align);
    push_padding(&mut attrs, n.padding.as_deref());
    push_unit(&mut attrs, "border-radius", n.border_radius.as_deref());
    push_unit(
        &mut attrs,
        "tb-border-radius",
        n.tb_border_radius.as_deref(),
    );
    push_opt(&mut attrs, "icon-width", n.icon_width.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    attrs.push((
        "thumbnails",
        match n.thumbnails {
            Thumbnails::Visible => "visible",
            Thumbnails::Hidden => "hidden",
            Thumbnails::Supported => "supported",
        }
        .into(),
    ));
    w.open_attrs("mj-carousel", &attrs);
    for img in &n.images {
        emit_carousel_image(w, t, mode, img)?;
    }
    w.close("mj-carousel");
    Ok(())
}

fn emit_carousel_image(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    n: &MjCarouselImage,
) -> anyhow::Result<()> {
    let src = rewrite_src(&n.src, t, mode)?;
    let mut attrs = vec![
        ("src", xml_escape_attr(&src)),
        ("alt", xml_escape_attr(&n.alt)),
    ];
    if let Some(href) = n.href.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        attrs.push(("href", xml_escape_attr(href)));
    }
    if let Some(thumb) = n
        .thumbnails_src
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        attrs.push((
            "thumbnails-src",
            xml_escape_attr(&rewrite_src(thumb, t, mode)?),
        ));
    }
    push_unit(&mut attrs, "border-radius", n.border_radius.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.self_close("mj-carousel-image", &attrs);
    Ok(())
}

fn emit_table(w: &mut Writer, n: &MjTable) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(&mut attrs, "font-size", n.font_size.as_deref());
    push_opt(&mut attrs, "font-family", n.font_family.as_deref());
    push_opt(&mut attrs, "line-height", n.line_height.as_deref());
    push_opt(&mut attrs, "color", n.color.as_deref());
    push_align(&mut attrs, n.align);
    push_opt(&mut attrs, "width", n.width.as_deref());
    push_opt(&mut attrs, "border", n.border.as_deref());
    push_padding(&mut attrs, n.padding.as_deref());
    push_opt(&mut attrs, "cellpadding", n.cellpadding.as_deref());
    push_opt(&mut attrs, "cellspacing", n.cellspacing.as_deref());
    push_opt(&mut attrs, "role", n.role.as_deref());
    push_opt(&mut attrs, "css-class", n.css_class.as_deref());
    w.open_attrs("mj-table", &attrs);
    let inner = if contains_ci(&n.content, "</mj-") {
        xml_escape_text(&n.content)
    } else {
        n.content.trim().to_string()
    };
    w.raw_indented(&inner);
    w.close("mj-table");
    Ok(())
}

fn emit_email_header(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    h: &EmailHeader,
) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(
        &mut attrs,
        "background-color",
        h.background_color.as_deref(),
    );
    w.open_attrs("mj-section", &attrs);
    w.open("mj-column");
    if !h.logo_src.trim().is_empty() {
        let src = rewrite_src(&h.logo_src, t, mode)?;
        let mut img = vec![
            ("src", xml_escape_attr(&src)),
            ("alt", xml_escape_attr(&h.logo_alt)),
            ("width", xml_escape_attr(&h.logo_width)),
            ("align", "left".into()),
        ];
        if let Some(href) = h
            .logo_href
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            img.push(("href", xml_escape_attr(href)));
        }
        w.self_close("mj-image", &img);
    }
    w.close("mj-column");
    w.close("mj-section");
    Ok(())
}

fn emit_email_hero(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    h: &EmailHero,
) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(
        &mut attrs,
        "background-color",
        h.background_color.as_deref(),
    );
    w.open_attrs("mj-section", &attrs);
    w.open("mj-column");
    if !h.image_src.trim().is_empty() {
        let src = rewrite_src(&h.image_src, t, mode)?;
        w.self_close(
            "mj-image",
            &[
                ("src", xml_escape_attr(&src)),
                ("alt", xml_escape_attr(&h.image_alt)),
            ],
        );
    }
    if !h.heading.trim().is_empty() {
        w.open_attrs(
            "mj-text",
            &[("font-size", "28px".into()), ("font-weight", "bold".into())],
        );
        w.raw_indented(&xml_escape_text(&h.heading));
        w.close("mj-text");
    }
    if !h.subheading.trim().is_empty() {
        w.open_attrs(
            "mj-text",
            &[("color", xml_escape_attr(&t.brand.text_color))],
        );
        w.raw_indented(&xml_escape_text(&h.subheading));
        w.close("mj-text");
    }
    w.close("mj-column");
    w.close("mj-section");
    Ok(())
}

fn emit_email_cta(
    w: &mut Writer,
    _t: &Template,
    _mode: &EmitMode,
    c: &EmailCta,
) -> anyhow::Result<()> {
    let mut attrs = Vec::new();
    push_opt(
        &mut attrs,
        "background-color",
        c.background_color.as_deref(),
    );
    w.open_attrs("mj-section", &attrs);
    w.open("mj-column");
    if !c.heading.trim().is_empty() {
        w.open_attrs(
            "mj-text",
            &[("font-size", "22px".into()), ("font-weight", "bold".into())],
        );
        w.raw_indented(&xml_escape_text(&c.heading));
        w.close("mj-text");
    }
    if !c.copy.trim().is_empty() {
        w.open("mj-text");
        w.raw_indented(&xml_escape_text(&c.copy));
        w.close("mj-text");
    }
    if !c.button_label.trim().is_empty() {
        w.open_attrs("mj-button", &[("href", xml_escape_attr(&c.button_href))]);
        w.raw_indented(&xml_escape_text(&c.button_label));
        w.close("mj-button");
    }
    w.close("mj-column");
    w.close("mj-section");
    Ok(())
}

fn emit_email_article(
    w: &mut Writer,
    t: &Template,
    mode: &EmitMode,
    a: &EmailArticle,
) -> anyhow::Result<()> {
    w.open("mj-section");
    let has_image = !a.image_src.trim().is_empty();
    let image_col = |w: &mut Writer| -> anyhow::Result<()> {
        w.open_attrs("mj-column", &[("width", "50%".into())]);
        let src = rewrite_src(&a.image_src, t, mode)?;
        w.self_close(
            "mj-image",
            &[
                ("src", xml_escape_attr(&src)),
                ("alt", xml_escape_attr(&a.image_alt)),
            ],
        );
        w.close("mj-column");
        Ok(())
    };
    let text_col = |w: &mut Writer, width: &str| {
        w.open_attrs("mj-column", &[("width", width.into())]);
        if !a.title.trim().is_empty() {
            w.open_attrs(
                "mj-text",
                &[("font-size", "20px".into()), ("font-weight", "bold".into())],
            );
            w.raw_indented(&xml_escape_text(&a.title));
            w.close("mj-text");
        }
        if !a.copy.trim().is_empty() {
            w.open("mj-text");
            w.raw_indented(&xml_escape_text(&a.copy));
            w.close("mj-text");
        }
        if !a.link_label.trim().is_empty() {
            w.open_attrs("mj-button", &[("href", xml_escape_attr(&a.link_href))]);
            w.raw_indented(&xml_escape_text(&a.link_label));
            w.close("mj-button");
        }
        w.close("mj-column");
    };

    if !has_image {
        text_col(w, "100%");
    } else {
        match a.image_position {
            ImagePosition::Top => {
                w.open("mj-column");
                let src = rewrite_src(&a.image_src, t, mode)?;
                w.self_close(
                    "mj-image",
                    &[
                        ("src", xml_escape_attr(&src)),
                        ("alt", xml_escape_attr(&a.image_alt)),
                    ],
                );
                w.close("mj-column");
                text_col(w, "100%");
            }
            ImagePosition::Left => {
                image_col(w)?;
                text_col(w, "50%");
            }
            ImagePosition::Right => {
                text_col(w, "50%");
                image_col(w)?;
            }
        }
    }
    w.close("mj-section");
    Ok(())
}

fn emit_email_footer(
    w: &mut Writer,
    t: &Template,
    _mode: &EmitMode,
    f: &EmailFooter,
) -> anyhow::Result<()> {
    w.open("mj-section");
    w.open("mj-column");
    w.self_close("mj-divider", &[]);
    if !f.social.is_empty() {
        w.open_attrs("mj-social", &[("mode", "horizontal".into())]);
        for el in &f.social {
            emit_social_element(w, el);
        }
        w.close("mj-social");
    }
    let mut lines: Vec<String> = Vec::new();
    if !f.company_name.trim().is_empty() {
        lines.push(xml_escape_text(&f.company_name));
    }
    for line in &f.address_lines {
        if !line.trim().is_empty() {
            lines.push(xml_escape_text(line));
        }
    }
    if !f.unsubscribe_href.trim().is_empty() {
        let label = if f.unsubscribe_label.trim().is_empty() {
            "Unsubscribe"
        } else {
            f.unsubscribe_label.trim()
        };
        lines.push(format!(
            "<a href=\"{}\">{}</a>",
            xml_escape_attr(&f.unsubscribe_href),
            xml_escape_text(label)
        ));
    }
    if let Some(copy) = f
        .copyright
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        lines.push(xml_escape_text(copy));
    }
    if !lines.is_empty() {
        w.open_attrs(
            "mj-text",
            &[
                ("font-size", "12px".into()),
                ("align", "center".into()),
                ("color", xml_escape_attr(&t.brand.text_color)),
            ],
        );
        w.raw_indented(&lines.join("<br />"));
        w.close("mj-text");
    }
    w.close("mj-column");
    w.close("mj-section");
    Ok(())
}

fn push_opt(attrs: &mut Vec<(&'static str, String)>, key: &'static str, value: Option<&str>) {
    if let Some(v) = value.map(str::trim).filter(|s| !s.is_empty()) {
        attrs.push((key, xml_escape_attr(v)));
    }
}

fn push_border(attrs: &mut Vec<(&'static str, String)>, value: Option<&str>, sides: Option<&str>) {
    push_border_named(attrs, "border", value, sides);
}

fn push_border_named(
    attrs: &mut Vec<(&'static str, String)>,
    shorthand: &'static str,
    value: Option<&str>,
    sides: Option<&str>,
) {
    let Some(v) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    let escaped = xml_escape_attr(v);
    let parsed = crate::border::Sides::from_storage(sides.unwrap_or(""))
        .unwrap_or(crate::border::Sides::ALL);
    for key in crate::border::mjml_attrs(shorthand, parsed) {
        attrs.push((key, escaped.clone()));
    }
}

fn push_padding(attrs: &mut Vec<(&'static str, String)>, value: Option<&str>) {
    push_padding_named(attrs, "padding", value);
}

fn push_padding_named(
    attrs: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<&str>,
) {
    if let Some(v) = value.map(str::trim).filter(|s| !s.is_empty()) {
        if let Ok(out) = crate::padding::normalize_padding(v) {
            attrs.push((key, xml_escape_attr(&out)));
        }
    }
}

fn push_bg_url(
    attrs: &mut Vec<(&'static str, String)>,
    url: Option<&str>,
    t: &Template,
    mode: &EmitMode,
) -> anyhow::Result<()> {
    if let Some(url) = url.map(str::trim).filter(|s| !s.is_empty()) {
        attrs.push((
            "background-url",
            xml_escape_attr(&rewrite_src(url, t, mode)?),
        ));
    }
    Ok(())
}

fn push_unit(attrs: &mut Vec<(&'static str, String)>, key: &'static str, value: Option<&str>) {
    if let Some(v) = value.map(str::trim).filter(|s| !s.is_empty()) {
        let out = crate::padding::normalize_padding(v)
            .or_else(|_| crate::padding::normalize_unit(v))
            .unwrap_or_else(|_| v.to_string());
        attrs.push((key, xml_escape_attr(&out)));
    }
}

fn push_align(attrs: &mut Vec<(&'static str, String)>, align: Option<Align>) {
    if let Some(a) = align {
        attrs.push((
            "align",
            match a {
                Align::Left => "left",
                Align::Center => "center",
                Align::Right => "right",
            }
            .into(),
        ));
    }
}

pub(crate) fn rewrite_src(src: &str, t: &Template, mode: &EmitMode) -> anyhow::Result<String> {
    let s = src.trim();
    if s.is_empty() {
        anyhow::bail!("empty image src");
    }
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("data:") || lower.starts_with("cid:") {
        anyhow::bail!("unsupported image URL scheme: {s}");
    }
    if lower.starts_with("https://") {
        return Ok(s.to_string());
    }
    if lower.starts_with("http://127.0.0.1:") || lower.starts_with("http://localhost:") {
        return match mode {
            EmitMode::Preview { .. } => Ok(s.to_string()),
            EmitMode::Export => anyhow::bail!("loopback image URL not allowed in export: {s}"),
        };
    }
    if lower.starts_with("http://") {
        anyhow::bail!("insecure image URL: {s}");
    }
    match mode {
        EmitMode::Preview { origin } => {
            let origin = origin.trim_end_matches('/');
            let rel = s.trim_start_matches('/');
            let rel = if rel.starts_with("images/") {
                rel.to_string()
            } else {
                format!("images/{rel}")
            };
            Ok(format!("{origin}/{rel}"))
        }
        EmitMode::Export => {
            if t.base_url.trim().is_empty() {
                anyhow::bail!("relative image '{s}' requires base_url");
            }
            let mut base = t.base_url.trim().to_string();
            if !base.ends_with('/') {
                base.push('/');
            }
            Ok(format!("{base}{}", s.trim_start_matches('/')))
        }
    }
}

fn xml_escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn xml_escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn emit_text_inner(content: &str) -> String {
    try_allowlisted(content).unwrap_or_else(|| xml_escape_text(content))
}

const ALLOWED_TAGS: &[&str] = &["br", "a", "b", "strong", "em", "span", "u"];

fn try_allowlisted(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = s;
    let mut stack: Vec<String> = Vec::new();
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix('<') {
            if let Some(end) = stripped.find('>') {
                let inner = &stripped[..end];
                rest = &stripped[end + 1..];
                let closing = inner.starts_with('/');
                let body = if closing { &inner[1..] } else { inner };
                let self_close = body.trim_end().ends_with('/');
                let body = body.trim_end().trim_end_matches('/').trim();
                if body.is_empty() {
                    return None;
                }
                let (name, attrs) = split_name_attrs(body)?;
                let name_l = name.to_ascii_lowercase();
                if !ALLOWED_TAGS.contains(&name_l.as_str()) {
                    return None;
                }
                if closing {
                    let last = stack.pop()?;
                    if last != name_l {
                        return None;
                    }
                    out.push_str("</");
                    out.push_str(&name_l);
                    out.push('>');
                } else {
                    out.push('<');
                    out.push_str(&name_l);
                    out.push_str(&attrs?);
                    if self_close || name_l == "br" {
                        out.push_str(" />");
                    } else {
                        out.push('>');
                        stack.push(name_l);
                    }
                }
            } else {
                return None;
            }
        } else {
            let next = rest.find('<').unwrap_or(rest.len());
            out.push_str(&xml_escape_text(&rest[..next]));
            rest = &rest[next..];
        }
    }
    if stack.is_empty() { Some(out) } else { None }
}

fn split_name_attrs(body: &str) -> Option<(String, Option<String>)> {
    let mut chars = body.char_indices();
    let mut name_end = body.len();
    for (i, c) in chars.by_ref() {
        if c.is_whitespace() {
            name_end = i;
            break;
        }
        if !c.is_ascii_alphabetic() {
            return None;
        }
    }
    let name = body[..name_end].to_string();
    let rest = body[name_end..].trim();
    if rest.is_empty() {
        return Some((name, Some(String::new())));
    }
    let mut attrs = String::new();
    let mut r = rest;
    while !r.is_empty() {
        let eq = r.find('=')?;
        let key = r[..eq].trim();
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
            return None;
        }
        r = r[eq + 1..].trim_start();
        let quote = r.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        r = &r[1..];
        let end = r.find(quote)?;
        let val = &r[..end];
        r = r[end + 1..].trim_start();
        attrs.push(' ');
        attrs.push_str(key);
        attrs.push_str("=\"");
        attrs.push_str(&xml_escape_attr(val));
        attrs.push('"');
    }
    Some((name, Some(attrs)))
}

struct Writer {
    buf: String,
    indent: usize,
}

impl Writer {
    fn new() -> Self {
        Self {
            buf: String::new(),
            indent: 0,
        }
    }

    fn pad(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }

    fn open(&mut self, tag: &str) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(tag);
        self.buf.push_str(">\n");
        self.indent += 1;
    }

    fn open_attrs(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(tag);
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            self.buf.push_str(v);
            self.buf.push('"');
        }
        self.buf.push_str(">\n");
        self.indent += 1;
    }

    fn close(&mut self, tag: &str) {
        self.indent = self.indent.saturating_sub(1);
        self.pad();
        self.buf.push_str("</");
        self.buf.push_str(tag);
        self.buf.push_str(">\n");
    }

    fn self_close(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(tag);
        for (k, v) in attrs {
            self.buf.push(' ');
            self.buf.push_str(k);
            self.buf.push_str("=\"");
            self.buf.push_str(v);
            self.buf.push('"');
        }
        self.buf.push_str(" />\n");
    }

    fn leaf_text(&mut self, tag: &str, text: &str) {
        self.pad();
        self.buf.push('<');
        self.buf.push_str(tag);
        self.buf.push('>');
        self.buf.push_str(text);
        self.buf.push_str("</");
        self.buf.push_str(tag);
        self.buf.push_str(">\n");
    }

    fn raw_indented(&mut self, text: &str) {
        for line in text.lines() {
            self.pad();
            self.buf.push_str(line);
            self.buf.push('\n');
        }
    }

    fn finish(mut self) -> String {
        if !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BodyNode, ColumnChild, EmailArticle, EmailCta, EmailFooter, EmailHeader, EmailHero,
        HeroMode, ImagePosition, MjButton, MjColumn, MjGroup, MjHero, MjSection, MjSocial,
        MjSocialElement, MjTable, MjText, MjWrapper, SectionChild, SocialNetwork, Template,
        WebFont,
    };

    fn export(t: &Template) -> String {
        emit_mjml(t, EmitMode::Export).unwrap()
    }

    fn preview(t: &Template) -> String {
        emit_mjml(
            t,
            EmitMode::Preview {
                origin: "http://127.0.0.1:48712".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn rewrite_src_unit_cases() {
        let mut t = Template::minimal();
        t.base_url = "https://cdn.example.com/".into();
        let preview = EmitMode::Preview {
            origin: "http://127.0.0.1:48712".into(),
        };
        assert!(rewrite_src("", &t, &EmitMode::Export).is_err());
        assert!(rewrite_src("data:image/png;base64,aaa", &t, &EmitMode::Export).is_err());
        assert!(rewrite_src("cid:ii_abc", &t, &EmitMode::Export).is_err());
        assert!(rewrite_src("http://127.0.0.1:1/x.png", &t, &EmitMode::Export).is_err());
        assert_eq!(
            rewrite_src("http://127.0.0.1:1/x.png", &t, &preview).unwrap(),
            "http://127.0.0.1:1/x.png"
        );
        assert_eq!(
            rewrite_src("https://cdn.example.com/a.png", &t, &EmitMode::Export).unwrap(),
            "https://cdn.example.com/a.png"
        );
        assert_eq!(
            rewrite_src("https://cdn.example.com/a.png", &t, &preview).unwrap(),
            "https://cdn.example.com/a.png"
        );
        assert_eq!(
            rewrite_src("images/a.png", &t, &preview).unwrap(),
            "http://127.0.0.1:48712/images/a.png"
        );
        assert_eq!(
            rewrite_src("images/a.png", &t, &EmitMode::Export).unwrap(),
            "https://cdn.example.com/images/a.png"
        );
    }

    #[test]
    fn unitless_padding_emits_px() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: Some("12 10 12 10".into()),
                inner_background_color: None,
                components: vec![ColumnChild::MjText(MjText {
                    content: "Hi".into(),
                    align: None,
                    font_size: None,
                    font_family: None,
                    color: None,
                    padding: Some("2".into()),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains(r#"padding="12px 10px 12px 10px""#), "{mjml}");
        assert!(mjml.contains(r#"padding="2px""#), "{mjml}");
        assert!(!mjml.contains(r#"padding="12 10 12 10""#), "{mjml}");
        assert!(!mjml.contains(r#"padding="2""#), "{mjml}");
    }

    #[test]
    fn p0_chrome_attrs_emit_when_set() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            border: Some("1px solid #000".into()),
            border_radius: Some("8px".into()),
            children: vec![SectionChild::MjColumn(MjColumn {
                border_radius: Some("4px".into()),
                components: vec![
                    ColumnChild::MjText(MjText {
                        content: "Hi".into(),
                        font_weight: Some("bold".into()),
                        font_style: Some("italic".into()),
                        line_height: Some("1.5".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjButton(MjButton {
                        content: "Go".into(),
                        href: "https://example.com".into(),
                        inner_padding: Some("8px 16px".into()),
                        font_size: Some("14px".into()),
                        border: Some("1px solid #333".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjImage(MjImage {
                        src: "https://example.com/a.png".into(),
                        alt: "A".into(),
                        border_radius: Some("6px".into()),
                        height: Some("120px".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjDivider(crate::model::MjDivider {
                        border_style: Some("dashed".into()),
                        width: Some("80%".into()),
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains(r#"border="1px solid #000""#), "{mjml}");
        assert!(mjml.contains(r#"border-radius="8px""#), "{mjml}");
        assert!(mjml.contains(r#"border-radius="4px""#), "{mjml}");
        assert!(mjml.contains(r#"font-weight="bold""#), "{mjml}");
        assert!(mjml.contains(r#"font-style="italic""#), "{mjml}");
        assert!(mjml.contains(r#"inner-padding="8px 16px""#), "{mjml}");
        assert!(mjml.contains(r#"border-style="dashed""#), "{mjml}");
        assert!(mjml.contains(r#"height="120px""#), "{mjml}");
    }

    #[test]
    fn border_sides_emit_per_side_attrs() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            border: Some("1px solid #000".into()),
            border_sides: Some("top,bottom".into()),
            children: vec![SectionChild::MjColumn(MjColumn {
                border: Some("2px dashed #333".into()),
                border_sides: Some("left".into()),
                inner_border: Some("1px solid #eee".into()),
                inner_border_sides: Some("right".into()),
                components: vec![
                    ColumnChild::MjButton(MjButton {
                        content: "Go".into(),
                        href: "https://example.com".into(),
                        border: Some("1px solid #333".into()),
                        border_sides: Some("top".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjImage(MjImage {
                        src: "https://example.com/a.png".into(),
                        alt: "A".into(),
                        border: Some("4px solid #f00".into()),
                        border_sides: Some("left,right".into()),
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains(r#"border-top="1px solid #000""#), "{mjml}");
        assert!(mjml.contains(r#"border-bottom="1px solid #000""#), "{mjml}");
        assert!(mjml.contains(r#"border-left="2px dashed #333""#), "{mjml}");
        assert!(
            mjml.contains(r#"inner-border-right="1px solid #eee""#),
            "{mjml}"
        );
        assert!(mjml.contains(r#"border-top="1px solid #333""#), "{mjml}");
        assert!(mjml.contains(r#"border-left="4px solid #f00""#), "{mjml}");
        assert!(mjml.contains(r#"border-right="4px solid #f00""#), "{mjml}");
        assert!(
            !mjml.contains(r#"<mj-section border="1px solid #000""#),
            "{mjml}"
        );
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"border_sides\":\"top,bottom\""), "{json}");
        let mut t2 = t.clone();
        if let BodyNode::MjSection(s) = &mut t2.body.nodes[0] {
            s.border_sides = None;
        }
        let all_sides = serde_json::to_string(&t2).unwrap();
        assert!(
            !all_sides.contains("\"border_sides\":\"top,bottom\""),
            "cleared sides should omit that value: {all_sides}"
        );
    }

    #[test]
    fn author_label_is_not_emitted() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            label: Some("hero".into()),
            children: vec![SectionChild::MjColumn(MjColumn::default())],
            ..Default::default()
        }));
        t.body.nodes.push(BodyNode::MjWrapper(MjWrapper {
            label: Some("footer".into()),
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(!mjml.contains("hero"), "{mjml}");
        assert!(!mjml.contains("footer"), "{mjml}");
        assert!(!mjml.contains("label="), "{mjml}");
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"label\":\"hero\""), "{json}");
        assert!(json.contains("\"label\":\"footer\""), "{json}");
    }

    #[test]
    fn p1_layout_attrs_emit_when_set() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.dir = "rtl".into();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            gutter: Some("4%".into()),
            background_url: Some("https://example.com/bg.png".into()),
            background_size: Some("cover".into()),
            background_repeat: Some("no-repeat".into()),
            direction: Some("rtl".into()),
            children: vec![SectionChild::MjColumn(MjColumn {
                vertical_align: Some("middle".into()),
                components: vec![
                    ColumnChild::MjButton(crate::model::MjButton {
                        content: "Go".into(),
                        href: "https://example.com".into(),
                        target: Some("_blank".into()),
                        height: Some("44px".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjImage(crate::model::MjImage {
                        src: "https://example.com/a.png".into(),
                        alt: "A".into(),
                        title: Some("Photo".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjSocial(crate::model::MjSocial {
                        icon_padding: Some("4px".into()),
                        color: Some("#333333".into()),
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains(r#"dir="rtl""#), "{mjml}");
        assert!(mjml.contains(r#"gutter="4%""#), "{mjml}");
        assert!(
            mjml.contains(r#"background-url="https://example.com/bg.png""#),
            "{mjml}"
        );
        assert!(mjml.contains(r#"background-size="cover""#), "{mjml}");
        assert!(mjml.contains(r#"direction="rtl""#), "{mjml}");
        assert!(mjml.contains(r#"vertical-align="middle""#), "{mjml}");
        assert!(mjml.contains(r#"target="_blank""#), "{mjml}");
        assert!(mjml.contains(r#"title="Photo""#), "{mjml}");
        assert!(mjml.contains(r#"icon-padding="4px""#), "{mjml}");
    }

    #[test]
    fn inner_padding_emits_on_button_social_and_hero() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjHero(MjHero {
            inner_padding: Some("8px 16px".into()),
            children: vec![ColumnChild::MjButton(MjButton {
                content: "Go".into(),
                href: "https://example.com".into(),
                inner_padding: Some("6px 12px".into()),
                ..Default::default()
            })],
            ..Default::default()
        }));
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            children: vec![SectionChild::MjColumn(MjColumn {
                components: vec![ColumnChild::MjSocial(MjSocial {
                    inner_padding: Some("4px".into()),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains(r#"inner-padding="8px 16px""#), "{mjml}");
        assert!(mjml.contains(r#"inner-padding="6px 12px""#), "{mjml}");
        assert!(mjml.contains(r#"inner-padding="4px""#), "{mjml}");
        assert!(mjml.contains("<mj-hero"), "{mjml}");
        assert!(mjml.contains("<mj-social"), "{mjml}");
        assert!(mjml.contains("<mj-button"), "{mjml}");
    }

    #[test]
    fn illegal_inner_padding_is_omitted_from_emit() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            children: vec![SectionChild::MjColumn(MjColumn {
                components: vec![ColumnChild::MjButton(MjButton {
                    content: "Go".into(),
                    href: "https://example.com".into(),
                    inner_padding: Some("-2px".into()),
                    padding: Some("10em".into()),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(!mjml.contains(r#"inner-padding="-2px""#), "{mjml}");
        assert!(!mjml.contains(r#"padding="10em""#), "{mjml}");
        assert!(
            mjml.contains(r#"inner-padding="12px 24px""#),
            "brand default still present\n{mjml}"
        );
    }

    #[test]
    fn p2_completeness_attrs_emit_when_set() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.css_class = Some("shell".into());
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            css_class: Some("hero-row".into()),
            children: vec![SectionChild::MjColumn(MjColumn {
                css_class: Some("col".into()),
                components: vec![
                    ColumnChild::MjText(MjText {
                        content: "Hi".into(),
                        letter_spacing: Some("0.5px".into()),
                        text_transform: Some("uppercase".into()),
                        css_class: Some("lede".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjButton(crate::model::MjButton {
                        content: "Go".into(),
                        href: "https://example.com".into(),
                        rel: Some("noopener".into()),
                        title: Some("Open".into()),
                        css_class: Some("cta".into()),
                        ..Default::default()
                    }),
                    ColumnChild::MjTable(MjTable {
                        content: "<tr><td>1</td></tr>".into(),
                        cellpadding: Some("4".into()),
                        role: Some("presentation".into()),
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains(r#"css-class="shell""#), "{mjml}");
        assert!(mjml.contains(r#"css-class="hero-row""#), "{mjml}");
        assert!(mjml.contains(r#"letter-spacing="0.5px""#), "{mjml}");
        assert!(mjml.contains(r#"text-transform="uppercase""#), "{mjml}");
        assert!(mjml.contains(r#"rel="noopener""#), "{mjml}");
        assert!(mjml.contains(r#"cellpadding="4""#), "{mjml}");
        assert!(mjml.contains(r#"role="presentation""#), "{mjml}");
        assert!(mjml.contains("<tr><td>1</td></tr>"), "{mjml}");
        assert!(
            !mjml.contains("<table><tr><td>1</td></tr></table>"),
            "mj-table must not wrap a nested <table>\n{mjml}"
        );
    }

    #[test]
    fn text_escaping_and_allowlist() {
        assert_eq!(emit_text_inner("Hello & Co"), "Hello &amp; Co");
        assert_eq!(emit_text_inner("<b>Hi</b>"), "<b>Hi</b>");
        assert_eq!(emit_text_inner("<script>"), "&lt;script&gt;");
        let escaped = emit_text_inner("</mj-text>oops");
        assert_eq!(escaped, "&lt;/mj-text&gt;oops");
        assert_eq!(escaped.matches("</mj-text>").count(), 0);
    }

    #[test]
    fn closer_cannot_close_mj_text_tag() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjText(MjText {
                    content: "</mj-text>oops".into(),
                    align: None,
                    font_size: None,
                    font_family: None,
                    color: None,
                    padding: None,
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert_eq!(mjml.matches("</mj-text>").count(), 1);
        assert!(mjml.contains("&lt;/mj-text&gt;oops"));
    }

    #[test]
    fn json_ld_is_pretty_value_in_head_not_raw_textarea() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.head.json_ld = "{\"@type\":\"EmailMessage\",\"@context\":\"https://schema.org\"}".into();
        let mjml = export(&t);
        let head_end = mjml.find("</mj-head>").unwrap();
        let raw_pos = mjml.find("<mj-raw>").unwrap();
        assert!(raw_pos < head_end, "json-ld mj-raw must live in mj-head");
        assert!(mjml.contains("<script type=\"application/ld+json\">"));
        assert!(mjml.contains("\"@type\": \"EmailMessage\""));
        assert!(!mjml.contains("{\"@type\":\"EmailMessage\""));
    }

    #[test]
    fn social_x_emits_twitter() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjSocial(MjSocial {
                    mode: SocialMode::Horizontal,
                    align: None,
                    icon_size: "32px".into(),
                    elements: vec![MjSocialElement {
                        name: SocialNetwork::X,
                        href: "https://x.com/acme".into(),
                        src: None,
                        ..Default::default()
                    }],
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains("name=\"twitter\""));
        assert!(!mjml.contains("name=\"x\""));
    }

    #[test]
    fn fonts_and_author_css_emit() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.head.fonts.push(WebFont {
            name: "Raleway".into(),
            href: "https://fonts.googleapis.com/css2?family=Raleway&display=swap".into(),
        });
        t.head.css = "a > b { color: red; }".into();
        t.head.css_inline = true;
        let mjml = export(&t);
        assert!(mjml.contains("<mj-font name=\"Raleway\" href=\"https://fonts.googleapis.com/css2?family=Raleway&amp;display=swap\" />"));
        assert!(mjml.contains("inline=\"inline\""));
        assert!(mjml.contains("a > b { color: red; }"));
        assert!(mjml.contains(".preheader { display:none"));
    }

    #[test]
    fn preview_rewrites_relative_images() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::EmailHeader(EmailHeader {
            logo_src: "logo.png".into(),
            logo_alt: "Logo".into(),
            logo_href: None,
            logo_width: "160px".into(),
            background_color: None,
        }));
        let mjml = preview(&t);
        assert!(mjml.contains("src=\"http://127.0.0.1:48712/images/logo.png\""));
    }

    #[test]
    fn kitchen_sink_is_deterministic_and_covers_blocks() {
        let t = kitchen_sink();
        let a = export(&t);
        let b = export(&t);
        assert_eq!(a, b);
        for needle in [
            "<mj-section",
            "<mj-wrapper",
            "<mj-hero",
            "<mj-group",
            "<mj-divider",
            "<mj-spacer",
            "<mj-table",
            "<tr><td>1</td></tr>",
            "<mj-navbar",
            "hamburger=\"hamburger\"",
            "<mj-navbar-link",
            "<mj-accordion",
            "<mj-accordion-element",
            "<mj-accordion-title",
            "<mj-accordion-text",
            "<mj-carousel",
            "thumbnails=\"hidden\"",
            "<mj-carousel-image",
            "font-size=\"28px\"",
            "Unsubscribe",
            "name=\"twitter\"",
        ] {
            assert!(a.contains(needle), "missing {needle}\n{a}");
        }
        assert!(!a.contains("mj-include"));
        assert_eq!(a.matches("<mj-raw>").count(), 1);
        assert!(a.contains("thumbnails-src=\"https://cdn.example.com/thumb.png\""));
        assert!(!a.contains("hamburger=\"true\""));
    }

    #[test]
    fn hamburger_false_omits_attr() {
        let mut t = Template::minimal();
        t.preheader.clear();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjNavbar(crate::model::MjNavbar {
                    hamburger: false,
                    ico_color: None,
                    base_url: None,
                    align: None,
                    padding: None,
                    links: vec![],
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let mjml = export(&t);
        assert!(mjml.contains("<mj-navbar"));
        assert!(!mjml.contains("hamburger="));
    }

    fn kitchen_sink() -> Template {
        let mut t = Template::minimal();
        t.base_url = "https://cdn.example.com/".into();
        t.head.fonts.push(WebFont {
            name: "Raleway".into(),
            href: "https://fonts.googleapis.com/css2?family=Raleway&display=swap".into(),
        });
        t.head.json_ld = r#"{"@context":"https://schema.org","@type":"EmailMessage"}"#.into();
        t.body.nodes = vec![
            BodyNode::EmailHeader(EmailHeader {
                logo_src: "https://cdn.example.com/logo.png".into(),
                logo_alt: "Logo".into(),
                logo_href: Some("https://example.com".into()),
                logo_width: "160px".into(),
                background_color: None,
            }),
            BodyNode::EmailHero(EmailHero {
                image_src: String::new(),
                image_alt: String::new(),
                heading: "You're in.".into(),
                subheading: "Welcome.".into(),
                background_color: None,
            }),
            BodyNode::EmailCta(EmailCta {
                heading: "Shop".into(),
                copy: "Now".into(),
                button_label: "Go".into(),
                button_href: "https://example.com".into(),
                background_color: None,
            }),
            BodyNode::EmailArticle(EmailArticle {
                image_src: "https://cdn.example.com/a.png".into(),
                image_alt: "A".into(),
                title: "Story".into(),
                copy: "Body".into(),
                link_label: "More".into(),
                link_href: "https://example.com".into(),
                image_position: ImagePosition::Left,
            }),
            BodyNode::MjWrapper(MjWrapper {
                background_color: None,
                padding: None,
                full_width: false,
                children: vec![BodyNode::MjSection(MjSection {
                    background_color: None,
                    padding: None,
                    full_width: false,
                    children: vec![SectionChild::MjGroup(MjGroup {
                        width: None,
                        background_color: None,
                        children: vec![MjColumn {
                            width: Some("100%".into()),
                            background_color: None,
                            padding: None,
                            inner_background_color: None,
                            components: vec![
                                ColumnChild::MjDivider(crate::model::MjDivider {
                                    border_color: None,
                                    border_width: None,
                                    padding: None,
                                    ..Default::default()
                                }),
                                ColumnChild::MjSpacer(crate::model::MjSpacer {
                                    height: "24px".into(),
                                    ..Default::default()
                                }),
                                ColumnChild::MjTable(MjTable {
                                    content: "<tr><td>1</td></tr>".into(),
                                    font_size: None,
                                    color: None,
                                    padding: None,
                                    ..Default::default()
                                }),
                                ColumnChild::MjNavbar(crate::model::MjNavbar {
                                    hamburger: true,
                                    ico_color: Some("#ffffff".into()),
                                    base_url: None,
                                    align: None,
                                    padding: None,
                                    links: vec![crate::model::MjNavbarLink {
                                        href: "https://example.com".into(),
                                        content: "Home".into(),
                                        color: None,
                                        padding: None,
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                }),
                                ColumnChild::MjAccordion(crate::model::MjAccordion {
                                    border: None,
                                    padding: None,
                                    elements: vec![crate::model::MjAccordionElement {
                                        title: "Why?".into(),
                                        content: "Because.".into(),
                                        background_color: None,
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                }),
                                ColumnChild::MjCarousel(crate::model::MjCarousel {
                                    align: None,
                                    padding: None,
                                    border_radius: None,
                                    thumbnails: crate::model::Thumbnails::Hidden,
                                    images: vec![crate::model::MjCarouselImage {
                                        src: "https://cdn.example.com/slide.png".into(),
                                        alt: "Slide".into(),
                                        href: None,
                                        thumbnails_src: Some(
                                            "https://cdn.example.com/thumb.png".into(),
                                        ),
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        }],
                        ..Default::default()
                    })],
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
                children: vec![ColumnChild::MjText(MjText {
                    content: "<b>Hi</b>".into(),
                    align: None,
                    font_size: None,
                    font_family: None,
                    color: None,
                    padding: None,
                    ..Default::default()
                })],
                ..Default::default()
            }),
            BodyNode::EmailFooter(EmailFooter {
                company_name: "Acme".into(),
                address_lines: vec!["1 Main".into()],
                unsubscribe_label: "Unsubscribe".into(),
                unsubscribe_href: "*|UNSUB|*".into(),
                social: vec![MjSocialElement {
                    name: SocialNetwork::X,
                    href: "https://x.com/acme".into(),
                    src: None,
                    ..Default::default()
                }],
                copyright: Some("© Acme".into()),
            }),
        ];
        t
    }
}
