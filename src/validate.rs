use std::path::Path;

use crate::model::{
    BodyNode, ColumnChild, EmailArticle, EmailFooter, EmailHeader, EmailHero, MjColumn, MjGroup,
    MjHero, MjSection, MjSocialElement, MjWrapper, SectionChild, SocialNetwork, Template,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ValidateReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidateReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Clone, Copy)]
struct ValidateOpts<'a> {
    export: bool,
    base_url: &'a str,
}

#[allow(dead_code)]
pub fn validate_template(t: &Template) -> ValidateReport {
    validate_inner(
        t,
        None,
        ValidateOpts {
            export: false,
            base_url: &t.base_url,
        },
    )
}

pub fn validate_template_with_root(t: &Template, root: Option<&Path>) -> ValidateReport {
    validate_inner(
        t,
        root,
        ValidateOpts {
            export: false,
            base_url: &t.base_url,
        },
    )
}

pub fn validate_template_for_export(t: &Template, root: Option<&Path>) -> ValidateReport {
    validate_inner(
        t,
        root,
        ValidateOpts {
            export: true,
            base_url: &t.base_url,
        },
    )
}

fn validate_inner(t: &Template, root: Option<&Path>, opts: ValidateOpts<'_>) -> ValidateReport {
    let mut report = ValidateReport::default();
    if t.version != 1 {
        report
            .errors
            .push(format!("version must be 1, got {}", t.version));
    }
    require_nonempty(&mut report, "name", &t.name);
    require_nonempty(&mut report, "subject", &t.subject);
    require_nonempty(&mut report, "head.title", &t.head.title);
    require_nonempty(&mut report, "lang", &t.lang);
    check_one_of(&mut report, "dir", &t.dir, &["auto", "ltr", "rtl"]);

    if !(320..=800).contains(&t.brand.content_width) {
        report.errors.push(format!(
            "brand.content_width must be 320..=800, got {}",
            t.brand.content_width
        ));
    }

    check_color(&mut report, "brand.text_color", &t.brand.text_color);
    check_color(
        &mut report,
        "brand.background_color",
        &t.brand.background_color,
    );
    check_color(
        &mut report,
        "brand.button_background",
        &t.brand.button_background,
    );
    check_color(&mut report, "brand.button_color", &t.brand.button_color);
    check_color(
        &mut report,
        "body.background_color",
        &t.body.background_color,
    );

    validate_fonts(t, &mut report);
    validate_json_ld(t, &mut report);
    validate_css(t, &mut report);

    let mut font_haystack = t.brand.font_family.clone();
    let mut relative_images = false;
    for node in &t.body.nodes {
        walk_body(
            node,
            &mut report,
            root,
            opts,
            &mut font_haystack,
            &mut relative_images,
        );
    }

    for font in &t.head.fonts {
        let name = font.name.trim();
        if !name.is_empty()
            && !font_haystack
                .to_ascii_lowercase()
                .contains(&name.to_ascii_lowercase())
        {
            report.warnings.push(format!(
                "font '{}' is registered but not used in brand.font_family or any node font_family",
                font.name
            ));
        }
    }

    if relative_images && t.base_url.trim().is_empty() && !opts.export {
        report
            .warnings
            .push("base_url is empty while relative images exist".to_string());
    }

    report
}

fn require_nonempty(report: &mut ValidateReport, field: &str, value: &str) {
    if value.trim().is_empty() {
        report.errors.push(format!("{field} is empty"));
    }
}

fn check_color(report: &mut ValidateReport, field: &str, value: &str) {
    let v = value.trim();
    if v.is_empty() {
        return;
    }
    if !is_hex_color(v) {
        report
            .errors
            .push(format!("{field} must be #RRGGBB, got '{value}'"));
    }
}

fn is_hex_color(value: &str) -> bool {
    let hex = value.trim().trim_start_matches('#');
    hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) && value.trim().starts_with('#')
}

fn validate_fonts(t: &Template, report: &mut ValidateReport) {
    let mut seen = Vec::new();
    for (i, font) in t.head.fonts.iter().enumerate() {
        if font.name.trim().is_empty() {
            report.errors.push(format!("head.fonts[{i}].name is empty"));
        }
        let key = font.name.trim().to_ascii_lowercase();
        if !key.is_empty() && seen.iter().any(|s: &String| s == &key) {
            report
                .errors
                .push(format!("duplicate font name '{}'", font.name));
        }
        seen.push(key);
        if !is_google_fonts_css_url(&font.href) {
            report.errors.push(format!(
                "head.fonts[{i}].href must be a Google Fonts CSS URL, got '{}'",
                font.href
            ));
        }
    }
}

fn is_google_fonts_css_url(href: &str) -> bool {
    let h = href.trim();
    h.starts_with("https://fonts.googleapis.com/css?")
        || h.starts_with("https://fonts.googleapis.com/css2?")
}

fn validate_json_ld(t: &Template, report: &mut ValidateReport) {
    let raw = t.head.json_ld.trim();
    if raw.is_empty() {
        return;
    }
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::Object(map)) => {
            let pretty = serde_json::to_string_pretty(&serde_json::Value::Object(map.clone()))
                .unwrap_or_default();
            if contains_ci(&pretty, "</script") || contains_ci(&pretty, "</mj-") {
                report
                    .errors
                    .push("head.json_ld pretty form contains a forbidden closer".to_string());
            }
            if !map.contains_key("@type") {
                report
                    .warnings
                    .push("head.json_ld has no @type at the root".to_string());
            }
        }
        Ok(serde_json::Value::Array(arr)) => {
            let pretty = serde_json::to_string_pretty(&serde_json::Value::Array(arr.clone()))
                .unwrap_or_default();
            if contains_ci(&pretty, "</script") || contains_ci(&pretty, "</mj-") {
                report
                    .errors
                    .push("head.json_ld pretty form contains a forbidden closer".to_string());
            }
            let any_type = arr.iter().any(|v| v.get("@type").is_some());
            if !any_type {
                report
                    .warnings
                    .push("head.json_ld array has no @type on any element".to_string());
            }
        }
        Ok(_) => {
            report
                .errors
                .push("head.json_ld must be a JSON object or array".to_string());
        }
        Err(e) => {
            report
                .errors
                .push(format!("head.json_ld is not valid JSON: {e}"));
        }
    }
}

fn validate_css(t: &Template, report: &mut ValidateReport) {
    let css = t.head.css.trim();
    if css.is_empty() {
        return;
    }
    if contains_ci(css, "</mj-style>") || contains_ci(css, "</mj-") {
        report
            .errors
            .push("head.css contains a forbidden MJML closer".to_string());
    }
    if contains_ci(css, "@import") {
        report
            .errors
            .push("head.css must not contain @import".to_string());
    }
    for url in css_urls(css) {
        if !url.starts_with("https://fonts.googleapis.com/") {
            report.errors.push(format!(
                "head.css url() must be https://fonts.googleapis.com/, got '{url}'"
            ));
        }
    }
}

fn css_urls(css: &str) -> Vec<String> {
    let lower = css.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut search = 0;
    while let Some(idx) = lower[search..].find("url(") {
        let start = search + idx + 4;
        let rest = &css[start..];
        let end = rest.find(')').unwrap_or(rest.len());
        let raw = rest[..end].trim().trim_matches(['\'', '"']);
        out.push(raw.to_string());
        search = start + end + 1;
        if search >= css.len() {
            break;
        }
    }
    out
}

fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn walk_body(
    node: &BodyNode,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    match node {
        BodyNode::MjSection(section) => {
            walk_section(section, report, root, opts, font_haystack, relative_images)
        }
        BodyNode::MjWrapper(wrapper) => {
            walk_wrapper(wrapper, report, root, opts, font_haystack, relative_images)
        }
        BodyNode::MjHero(hero) => {
            walk_hero(hero, report, root, opts, font_haystack, relative_images)
        }
        BodyNode::EmailHeader(h) => walk_email_header(h, report, root, opts, relative_images),
        BodyNode::EmailHero(h) => walk_email_hero(h, report, root, opts, relative_images),
        BodyNode::EmailCta(c) => {
            check_color(
                report,
                "email-cta.background_color",
                opt(&c.background_color),
            );
            if c.button_href.trim().is_empty() {
                report
                    .errors
                    .push("email-cta.button_href is empty".to_string());
            }
        }
        BodyNode::EmailArticle(a) => walk_email_article(a, report, root, opts, relative_images),
        BodyNode::EmailFooter(f) => walk_email_footer(f, report),
    }
}

fn opt(v: &Option<String>) -> &str {
    v.as_deref().unwrap_or("")
}

fn check_border_sides(report: &mut ValidateReport, field: &str, value: &Option<String>) {
    let Some(v) = value.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    if let Err(e) = crate::border::Sides::from_storage(v) {
        report.errors.push(format!("{field} {e}"));
    }
}

fn check_padding(report: &mut ValidateReport, field: &str, value: &Option<String>) {
    let Some(v) = value.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    if crate::padding::normalize_padding(v).is_err() {
        report.errors.push(format!(
            "{field} must be {}, got '{v}'",
            crate::padding::RULE
        ));
    }
}

/// MJML sizes the inner `<a>` as `width - innerPadding.left/right - borders`.
/// Brand `mj-attributes` still apply `12px 24px` when the node omits inner-padding.
fn check_button_inner_padding_vs_width(report: &mut ValidateReport, btn: &crate::model::MjButton) {
    let Some(width) = btn
        .width
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    let Some(width_px) = width.strip_suffix("px").and_then(|n| n.parse::<f64>().ok()) else {
        return;
    };
    let inner = btn
        .inner_padding
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(crate::emit::BRAND_BUTTON_INNER_PADDING);
    let Some(inner_h) = crate::padding::horizontal_px(inner) else {
        return;
    };
    let border_h = btn
        .border
        .as_deref()
        .map(|b| crate::border::horizontal_px(b, btn.border_sides.as_deref()))
        .unwrap_or(0.0);
    if width_px - inner_h - border_h <= 0.0 {
        report.errors.push(format!(
            "mj-button inner-padding + border leave no room inside width {width} (MJML collapses the button)"
        ));
    }
}

fn check_one_of(report: &mut ValidateReport, field: &str, value: &str, allowed: &[&str]) {
    let v = value.trim();
    if v.is_empty() {
        return;
    }
    if !allowed.iter().any(|a| *a == v) {
        report.errors.push(format!(
            "{field} must be one of {}, got '{value}'",
            allowed.join("|")
        ));
    }
}

fn check_opt_one_of(
    report: &mut ValidateReport,
    field: &str,
    value: &Option<String>,
    allowed: &[&str],
) {
    check_one_of(report, field, opt(value), allowed);
}

fn check_unit(report: &mut ValidateReport, field: &str, value: &Option<String>) {
    let Some(v) = value.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    if crate::padding::normalize_padding(v).is_ok() || crate::padding::normalize_unit(v).is_ok() {
        return;
    }
    report.errors.push(format!(
        "{field} must be {}, got '{v}'",
        crate::padding::UNIT_RULE
    ));
}

fn walk_section(
    section: &MjSection,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "mj-section.background_color",
        opt(&section.background_color),
    );
    check_padding(report, "mj-section.padding", &section.padding);
    check_border_sides(report, "mj-section.border_sides", &section.border_sides);
    check_unit(report, "mj-section.border_radius", &section.border_radius);
    check_unit(report, "mj-section.gutter", &section.gutter);
    check_opt_one_of(
        report,
        "mj-section.direction",
        &section.direction,
        &["ltr", "rtl"],
    );
    check_opt_one_of(
        report,
        "mj-section.background_repeat",
        &section.background_repeat,
        &["repeat", "no-repeat"],
    );
    if let Some(url) = &section.background_url {
        check_optional_image(
            report,
            "mj-section.background_url",
            url,
            root,
            opts,
            relative_images,
        );
    }
    if section.children.is_empty() {
        report.errors.push("mj-section has no children".to_string());
    }
    for child in &section.children {
        match child {
            SectionChild::MjColumn(col) => {
                walk_column(col, report, root, opts, font_haystack, relative_images)
            }
            SectionChild::MjGroup(group) => {
                walk_group(group, report, root, opts, font_haystack, relative_images)
            }
        }
    }
}

fn walk_wrapper(
    wrapper: &MjWrapper,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "mj-wrapper.background_color",
        opt(&wrapper.background_color),
    );
    check_padding(report, "mj-wrapper.padding", &wrapper.padding);
    check_border_sides(report, "mj-wrapper.border_sides", &wrapper.border_sides);
    check_unit(report, "mj-wrapper.border_radius", &wrapper.border_radius);
    check_unit(report, "mj-wrapper.gap", &wrapper.gap);
    check_opt_one_of(
        report,
        "mj-wrapper.background_repeat",
        &wrapper.background_repeat,
        &["repeat", "no-repeat"],
    );
    if let Some(url) = &wrapper.background_url {
        check_optional_image(
            report,
            "mj-wrapper.background_url",
            url,
            root,
            opts,
            relative_images,
        );
    }
    for child in &wrapper.children {
        match child {
            BodyNode::MjSection(_) | BodyNode::MjHero(_) => {
                walk_body(child, report, root, opts, font_haystack, relative_images)
            }
            _ => report
                .errors
                .push("mj-wrapper may only contain mj-section or mj-hero".to_string()),
        }
    }
}

fn walk_group(
    group: &MjGroup,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "mj-group.background_color",
        opt(&group.background_color),
    );
    check_opt_one_of(
        report,
        "mj-group.direction",
        &group.direction,
        &["ltr", "rtl"],
    );
    check_opt_one_of(
        report,
        "mj-group.vertical_align",
        &group.vertical_align,
        &["top", "middle", "bottom"],
    );
    if group.children.is_empty() {
        report.errors.push("mj-group has no columns".to_string());
    }
    for col in &group.children {
        walk_column(col, report, root, opts, font_haystack, relative_images);
    }
}

fn walk_column(
    col: &MjColumn,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "mj-column.background_color",
        opt(&col.background_color),
    );
    check_color(
        report,
        "mj-column.inner_background_color",
        opt(&col.inner_background_color),
    );
    check_padding(report, "mj-column.padding", &col.padding);
    check_border_sides(report, "mj-column.border_sides", &col.border_sides);
    check_unit(report, "mj-column.border_radius", &col.border_radius);
    check_border_sides(
        report,
        "mj-column.inner_border_sides",
        &col.inner_border_sides,
    );
    check_opt_one_of(
        report,
        "mj-column.vertical_align",
        &col.vertical_align,
        &["top", "middle", "bottom"],
    );
    check_unit(
        report,
        "mj-column.inner_border_radius",
        &col.inner_border_radius,
    );
    for child in &col.components {
        walk_column_child(child, report, root, opts, font_haystack, relative_images);
    }
}

fn walk_hero(
    hero: &MjHero,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "mj-hero.background_color",
        opt(&hero.background_color),
    );
    check_padding(report, "mj-hero.padding", &hero.padding);
    check_padding(report, "mj-hero.inner_padding", &hero.inner_padding);
    check_unit(report, "mj-hero.border_radius", &hero.border_radius);
    check_opt_one_of(
        report,
        "mj-hero.vertical_align",
        &hero.vertical_align,
        &["top", "middle", "bottom"],
    );
    if let Some(url) = &hero.background_url {
        check_optional_image(
            report,
            "mj-hero.background_url",
            url,
            root,
            opts,
            relative_images,
        );
    }
    for child in &hero.children {
        walk_column_child(child, report, root, opts, font_haystack, relative_images);
    }
}

fn walk_column_child(
    child: &ColumnChild,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    font_haystack: &mut String,
    relative_images: &mut bool,
) {
    match child {
        ColumnChild::MjText(text) => {
            check_color(report, "mj-text.color", opt(&text.color));
            check_padding(report, "mj-text.padding", &text.padding);
            check_unit(report, "mj-text.line_height", &text.line_height);
            if let Some(ff) = &text.font_family {
                font_haystack.push(' ');
                font_haystack.push_str(ff);
            }
            if contains_ci(&text.content, "</mj-") {
                report
                    .errors
                    .push("mj-text.content contains </mj-".to_string());
            }
        }
        ColumnChild::MjButton(btn) => {
            check_color(
                report,
                "mj-button.background_color",
                opt(&btn.background_color),
            );
            check_color(report, "mj-button.color", opt(&btn.color));
            check_padding(report, "mj-button.padding", &btn.padding);
            check_padding(report, "mj-button.inner_padding", &btn.inner_padding);
            check_button_inner_padding_vs_width(report, btn);
            check_border_sides(report, "mj-button.border_sides", &btn.border_sides);
            check_unit(report, "mj-button.border_radius", &btn.border_radius);
            check_unit(report, "mj-button.font_size", &btn.font_size);
            check_unit(report, "mj-button.height", &btn.height);
            check_opt_one_of(
                report,
                "mj-button.target",
                &btn.target,
                &["_blank", "_self"],
            );
            if let Some(ff) = &btn.font_family {
                font_haystack.push(' ');
                font_haystack.push_str(ff);
            }
            if btn.href.trim().is_empty() {
                report.errors.push("mj-button.href is empty".to_string());
            }
        }
        ColumnChild::MjImage(img) => {
            check_required_image(
                report,
                "mj-image.src",
                &img.src,
                root,
                opts,
                relative_images,
            );
            check_padding(report, "mj-image.padding", &img.padding);
            check_border_sides(report, "mj-image.border_sides", &img.border_sides);
            check_unit(report, "mj-image.border_radius", &img.border_radius);
            check_unit(report, "mj-image.height", &img.height);
            check_opt_one_of(report, "mj-image.target", &img.target, &["_blank", "_self"]);
            if img.alt.trim().is_empty() {
                report.errors.push("mj-image.alt is empty".to_string());
            }
        }
        ColumnChild::MjDivider(d) => {
            check_color(report, "mj-divider.border_color", opt(&d.border_color));
            check_padding(report, "mj-divider.padding", &d.padding);
            check_unit(report, "mj-divider.border_width", &d.border_width);
            check_unit(report, "mj-divider.width", &d.width);
        }
        ColumnChild::MjSpacer(sp) => {
            check_padding(report, "mj-spacer.padding", &sp.padding);
        }
        ColumnChild::MjSocial(social) => {
            check_padding(report, "mj-social.padding", &social.padding);
            check_padding(report, "mj-social.icon_padding", &social.icon_padding);
            check_padding(report, "mj-social.inner_padding", &social.inner_padding);
            check_unit(report, "mj-social.border_radius", &social.border_radius);
            check_color(report, "mj-social.color", opt(&social.color));
            for el in &social.elements {
                check_social_element(el, report);
            }
        }
        ColumnChild::MjTable(table) => {
            check_color(report, "mj-table.color", opt(&table.color));
            check_padding(report, "mj-table.padding", &table.padding);
            check_unit(report, "mj-table.line_height", &table.line_height);
            check_opt_one_of(
                report,
                "mj-table.role",
                &table.role,
                &["none", "presentation"],
            );
            if contains_ci(&table.content, "</mj-") {
                report
                    .errors
                    .push("mj-table.content contains </mj-".to_string());
            }
            if !is_table_inner_fragment(&table.content) {
                report.errors.push(
                    "mj-table.content must be table rows (<tr>…</tr>), not a wrapping <table>"
                        .to_string(),
                );
            }
        }
        ColumnChild::MjNavbar(nav) => walk_navbar(nav, report),
        ColumnChild::MjAccordion(acc) => walk_accordion(acc, report),
        ColumnChild::MjCarousel(car) => walk_carousel(car, report, root, opts, relative_images),
    }
}

fn walk_navbar(nav: &crate::model::MjNavbar, report: &mut ValidateReport) {
    check_color(report, "mj-navbar.ico_color", opt(&nav.ico_color));
    check_padding(report, "mj-navbar.padding", &nav.padding);
    if nav.links.is_empty() {
        report
            .warnings
            .push("mj-navbar has no mj-navbar-link children".to_string());
    }
    for link in &nav.links {
        if link.href.trim().is_empty() {
            report
                .errors
                .push("mj-navbar-link.href is empty".to_string());
        }
        check_color(report, "mj-navbar-link.color", opt(&link.color));
        check_padding(report, "mj-navbar-link.padding", &link.padding);
        check_unit(report, "mj-navbar-link.font_size", &link.font_size);
    }
}

fn walk_accordion(acc: &crate::model::MjAccordion, report: &mut ValidateReport) {
    check_padding(report, "mj-accordion.padding", &acc.padding);
    check_opt_one_of(
        report,
        "mj-accordion.icon_position",
        &acc.icon_position,
        &["left", "right"],
    );
    if acc.elements.is_empty() {
        report
            .warnings
            .push("mj-accordion has no mj-accordion-element children".to_string());
    }
    for el in &acc.elements {
        if el.title.trim().is_empty() {
            report
                .errors
                .push("mj-accordion-element.title is empty".to_string());
        }
        check_color(
            report,
            "mj-accordion-element.background_color",
            opt(&el.background_color),
        );
        if contains_ci(&el.content, "</mj-") {
            report
                .errors
                .push("mj-accordion-element.content contains </mj-".to_string());
        }
    }
}

fn walk_carousel(
    car: &crate::model::MjCarousel,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    check_padding(report, "mj-carousel.padding", &car.padding);
    check_unit(report, "mj-carousel.border_radius", &car.border_radius);
    check_unit(
        report,
        "mj-carousel.tb_border_radius",
        &car.tb_border_radius,
    );
    if car.images.is_empty() {
        report
            .warnings
            .push("mj-carousel has no mj-carousel-image children".to_string());
    }
    for img in &car.images {
        check_required_image(
            report,
            "mj-carousel-image.src",
            &img.src,
            root,
            opts,
            relative_images,
        );
        if img.alt.trim().is_empty() {
            report
                .errors
                .push("mj-carousel-image.alt is empty".to_string());
        }
        if let Some(thumb) = &img.thumbnails_src {
            check_optional_image(
                report,
                "mj-carousel-image.thumbnails_src",
                thumb,
                root,
                opts,
                relative_images,
            );
        }
    }
}

fn check_social_element(el: &MjSocialElement, report: &mut ValidateReport) {
    if el.href.trim().is_empty() {
        report
            .errors
            .push("mj-social-element.href is empty".to_string());
    }
    if el.name == SocialNetwork::Web && el.src.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
    {
        report
            .warnings
            .push("social web element has no icon src".to_string());
    }
    check_color(
        report,
        "mj-social-element.background_color",
        opt(&el.background_color),
    );
    check_padding(report, "mj-social-element.padding", &el.padding);
}

fn walk_email_header(
    h: &EmailHeader,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "email-header.background_color",
        opt(&h.background_color),
    );
    check_optional_image(
        report,
        "email-header.logo_src",
        &h.logo_src,
        root,
        opts,
        relative_images,
    );
    if !h.logo_src.trim().is_empty() && h.logo_alt.trim().is_empty() {
        report
            .errors
            .push("email-header.logo_alt is required when logo_src is set".to_string());
    }
}

fn walk_email_hero(
    h: &EmailHero,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    check_color(
        report,
        "email-hero.background_color",
        opt(&h.background_color),
    );
    check_optional_image(
        report,
        "email-hero.image_src",
        &h.image_src,
        root,
        opts,
        relative_images,
    );
    if !h.image_src.trim().is_empty() && h.image_alt.trim().is_empty() {
        report
            .errors
            .push("email-hero.image_alt is required when image_src is set".to_string());
    }
}

fn walk_email_article(
    a: &EmailArticle,
    report: &mut ValidateReport,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    check_optional_image(
        report,
        "email-article.image_src",
        &a.image_src,
        root,
        opts,
        relative_images,
    );
    if !a.image_src.trim().is_empty() && a.image_alt.trim().is_empty() {
        report
            .errors
            .push("email-article.image_alt is required when image_src is set".to_string());
    }
}

fn walk_email_footer(f: &EmailFooter, report: &mut ValidateReport) {
    let has_address = f.address_lines.iter().any(|l| !l.trim().is_empty());
    if !has_address {
        report
            .errors
            .push("email-footer.address_lines is empty".to_string());
    }
    for el in &f.social {
        check_social_element(el, report);
    }
    if has_address && f.unsubscribe_href.trim().is_empty() && !f.unsubscribe_label.trim().is_empty()
    {
        report
            .warnings
            .push("marketing mail usually needs an unsubscribe link".to_string());
    }
}

fn check_required_image(
    report: &mut ValidateReport,
    field: &str,
    src: &str,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    if src.trim().is_empty() {
        report.errors.push(format!("{field} is empty"));
        return;
    }
    check_image_src(report, field, src, root, opts, relative_images);
}

fn check_optional_image(
    report: &mut ValidateReport,
    field: &str,
    src: &str,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    if src.trim().is_empty() {
        return;
    }
    check_image_src(report, field, src, root, opts, relative_images);
}

fn check_image_src(
    report: &mut ValidateReport,
    field: &str,
    src: &str,
    root: Option<&Path>,
    opts: ValidateOpts<'_>,
    relative_images: &mut bool,
) {
    let s = src.trim();
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("data:") || lower.starts_with("cid:") {
        report
            .errors
            .push(format!("{field} uses unsupported scheme: {s}"));
        return;
    }
    if lower.starts_with("https://") {
        return;
    }
    if lower.starts_with("http://") {
        if opts.export {
            report
                .errors
                .push(format!("{field} must be https:// in export, got {s}"));
        } else if lower.starts_with("http://127.0.0.1:") || lower.starts_with("http://localhost:") {
            // Preview loopback is allowed outside export; still not a local file.
        } else {
            report
                .errors
                .push(format!("{field} uses insecure http:// URL: {s}"));
        }
        return;
    }
    *relative_images = true;
    if opts.export {
        let base = opts.base_url.trim();
        if !base.starts_with("https://") {
            report.errors.push(format!(
                "{field} is relative and requires an https:// base_url in export"
            ));
        }
    }
    if let Some(root) = root {
        if !local_image_exists(root, s) {
            report.errors.push(format!("Missing local image: {s}"));
        }
    }
}

fn local_image_exists(root: &Path, src: &str) -> bool {
    let rel = src.trim_start_matches('/');
    let candidates = if rel.starts_with("images/") {
        vec![root.join(rel)]
    } else {
        vec![root.join("images").join(rel), root.join(rel)]
    };
    candidates.iter().any(|p| p.is_file())
}

/// `mj-table` already emits `<table>`. Inner content must be row markup
/// (`<tr>` / `<td>` / `<th>`, optional thead/tbody/tfoot). A wrapping
/// `<table>…</table>` is illegal: browsers hoist it out of the parent table.
fn is_table_inner_fragment(content: &str) -> bool {
    let t = content.trim();
    if t.is_empty() {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    if starts_with_table_tag(&lower) {
        return false;
    }
    lower.contains("<tr") && lower.contains("</tr>")
}

fn starts_with_table_tag(lower: &str) -> bool {
    let Some(rest) = lower.strip_prefix("<table") else {
        return false;
    };
    rest.is_empty()
        || rest.starts_with('>')
        || rest.starts_with('/')
        || rest.starts_with(|c: char| c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BodyNode, ColumnChild, EmailFooter, EmailHero, MjButton, MjColumn, MjImage, MjSection,
        MjTable, MjText, Template, WebFont,
    };
    use std::fs;

    fn report_has(report: &ValidateReport, needle: &str) -> bool {
        report
            .errors
            .iter()
            .chain(report.warnings.iter())
            .any(|s| s.contains(needle))
    }

    #[test]
    fn minimal_template_passes() {
        let t = Template::minimal();
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }

    #[test]
    fn empty_name_is_error() {
        let mut t = Template::minimal();
        t.name.clear();
        let r = validate_template(&t);
        assert!(report_has(&r, "name is empty"));
    }

    #[test]
    fn content_width_out_of_range() {
        let mut t = Template::minimal();
        t.brand.content_width = 1000;
        let r = validate_template(&t);
        assert!(report_has(&r, "content_width"));
    }

    #[test]
    fn navbar_link_empty_href_is_error() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
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
                    links: vec![crate::model::MjNavbarLink {
                        href: String::new(),
                        content: "Home".into(),
                        color: None,
                        padding: None,
                        ..Default::default()
                    }],
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-navbar-link.href is empty"));
    }

    #[test]
    fn data_src_is_error() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjImage(MjImage {
                    src: "data:image/png;base64,aaa".to_string(),
                    alt: "x".to_string(),
                    href: None,
                    width: None,
                    align: None,
                    fluid_on_mobile: true,
                    padding: None,
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let r = validate_template(&t);
        assert!(report_has(&r, "unsupported scheme"));
    }

    #[test]
    fn empty_mj_image_src_is_error() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjImage(MjImage {
                    src: "  ".to_string(),
                    alt: "x".to_string(),
                    href: None,
                    width: None,
                    align: None,
                    fluid_on_mobile: true,
                    padding: None,
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-image.src is empty"));
    }

    #[test]
    fn text_only_hero_passes() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailHero(EmailHero {
            image_src: String::new(),
            image_alt: String::new(),
            heading: "Hi".to_string(),
            subheading: String::new(),
            background_color: None,
        }));
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
    }

    #[test]
    fn hero_with_src_needs_alt() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailHero(EmailHero {
            image_src: "https://example.com/a.png".to_string(),
            image_alt: String::new(),
            heading: "Hi".to_string(),
            subheading: String::new(),
            background_color: None,
        }));
        let r = validate_template(&t);
        assert!(report_has(&r, "image_alt"));
    }

    #[test]
    fn mj_text_closer_is_error() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjText(MjText {
                    content: "</mj-text>oops".to_string(),
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
        let r = validate_template(&t);
        assert!(report_has(&r, "</mj-"));
    }

    #[test]
    fn google_font_href_must_match() {
        let mut t = Template::minimal();
        t.head.fonts.push(WebFont {
            name: "Inter".to_string(),
            href: "https://example.com/inter.css".to_string(),
        });
        let r = validate_template(&t);
        assert!(report_has(&r, "Google Fonts"));
    }

    #[test]
    fn unused_font_is_warning() {
        let mut t = Template::minimal();
        t.head.fonts.push(WebFont {
            name: "Raleway".to_string(),
            href: "https://fonts.googleapis.com/css2?family=Raleway&display=swap".to_string(),
        });
        let r = validate_template(&t);
        assert!(r.ok());
        assert!(report_has(&r, "not used"));
    }

    #[test]
    fn json_ld_must_be_object_or_array() {
        let mut t = Template::minimal();
        t.head.json_ld = "\"hello\"".to_string();
        let r = validate_template(&t);
        assert!(report_has(&r, "object or array"));
    }

    #[test]
    fn json_ld_object_without_type_warns() {
        let mut t = Template::minimal();
        t.head.json_ld = r#"{"@context":"https://schema.org"}"#.to_string();
        let r = validate_template(&t);
        assert!(r.ok());
        assert!(report_has(&r, "@type"));
    }

    #[test]
    fn css_import_is_error() {
        let mut t = Template::minimal();
        t.head.css = "@import url('https://fonts.googleapis.com/css2?family=X');".to_string();
        let r = validate_template(&t);
        assert!(report_has(&r, "@import"));
    }

    #[test]
    fn css_offhost_url_is_error() {
        let mut t = Template::minimal();
        t.head.css = "body { background: url('https://evil.example/x.png'); }".to_string();
        let r = validate_template(&t);
        assert!(report_has(&r, "url()"));
    }

    #[test]
    fn transactional_footer_is_clean() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailFooter(EmailFooter {
            company_name: "Acme".to_string(),
            address_lines: vec!["1 Main".to_string()],
            unsubscribe_label: String::new(),
            unsubscribe_href: String::new(),
            social: vec![],
            copyright: None,
        }));
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }

    #[test]
    fn marketing_footer_missing_unsub_warns() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailFooter(EmailFooter {
            company_name: "Acme".to_string(),
            address_lines: vec!["1 Main".to_string()],
            unsubscribe_label: "Unsubscribe".to_string(),
            unsubscribe_href: String::new(),
            social: vec![],
            copyright: None,
        }));
        let r = validate_template(&t);
        assert!(r.ok());
        assert!(report_has(&r, "unsubscribe"));
    }

    #[test]
    fn empty_footer_address_is_error() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailFooter(EmailFooter {
            company_name: "Acme".to_string(),
            address_lines: vec!["  ".to_string()],
            unsubscribe_label: String::new(),
            unsubscribe_href: String::new(),
            social: vec![],
            copyright: None,
        }));
        let r = validate_template(&t);
        assert!(report_has(&r, "address_lines"));
    }

    #[test]
    fn missing_local_image_with_root() {
        let dir = std::env::temp_dir().join(format!("dd_emailforge_val_{}", std::process::id()));
        fs::create_dir_all(dir.join("images")).unwrap();
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjImage(MjImage {
                    src: "images/missing.png".to_string(),
                    alt: "x".to_string(),
                    href: None,
                    width: None,
                    align: None,
                    fluid_on_mobile: true,
                    padding: None,
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let r = validate_template_with_root(&t, Some(&dir));
        assert!(report_has(&r, "Missing local image"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_rejects_loopback_image() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjImage(MjImage {
                    src: "http://127.0.0.1:9/x.png".to_string(),
                    alt: "x".to_string(),
                    href: None,
                    width: None,
                    align: None,
                    fluid_on_mobile: true,
                    padding: None,
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        let r = validate_template_for_export(&t, None);
        assert!(report_has(&r, "https://"));
    }

    fn section_with_column_padding(padding: Option<String>) -> Template {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                width: None,
                background_color: None,
                padding,
                inner_background_color: None,
                components: Vec::new(),
                ..Default::default()
            })],
            ..Default::default()
        }));
        t
    }

    fn template_with_table(content: &str) -> Template {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                components: vec![ColumnChild::MjTable(MjTable {
                    content: content.to_string(),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        t
    }

    #[test]
    fn table_inner_fragment_shape() {
        assert!(!is_table_inner_fragment(""));
        assert!(!is_table_inner_fragment("   "));
        assert!(!is_table_inner_fragment("hello"));
        assert!(!is_table_inner_fragment(
            "<table><tr><td></td></tr></table>"
        ));
        assert!(!is_table_inner_fragment(
            "<TABLE border=\"0\"><tr><td>x</td></tr></TABLE>"
        ));
        assert!(is_table_inner_fragment("<tr><td></td></tr>"));
        assert!(is_table_inner_fragment(
            "<thead><tr><th>H</th></tr></thead><tbody><tr><td>B</td></tr></tbody>"
        ));
        assert!(is_table_inner_fragment(
            "<tr><td><table><tr><td>n</td></tr></table></td></tr>"
        ));
    }

    #[test]
    fn mj_table_wrapping_table_is_error() {
        let t = template_with_table("<table><tr><td></td></tr></table>");
        let r = validate_template(&t);
        assert!(report_has(&r, "wrapping <table>"));
    }

    #[test]
    fn mj_table_row_fragment_passes() {
        let t = template_with_table("<tr><td></td></tr>");
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
    }

    #[test]
    fn mj_table_empty_content_is_error() {
        let t = template_with_table("");
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-table.content"));
    }

    #[test]
    fn unitless_padding_is_accepted() {
        let t = section_with_column_padding(Some("12 10 12 10".into()));
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
    }

    #[test]
    fn padding_with_units_passes() {
        let t = section_with_column_padding(Some("10px 20%".into()));
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
    }

    #[test]
    fn padding_rejects_unknown_units() {
        let t = section_with_column_padding(Some("10em".into()));
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-column.padding"));
        assert!(report_has(&r, "px or %"));
    }

    #[test]
    fn padding_rejects_five_values() {
        let t = section_with_column_padding(Some("1px 2px 3px 4px 5px".into()));
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-column.padding"));
    }

    fn template_with_button(btn: MjButton) -> Template {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            children: vec![crate::model::SectionChild::MjColumn(MjColumn {
                components: vec![ColumnChild::MjButton(btn)],
                ..Default::default()
            })],
            ..Default::default()
        }));
        t
    }

    #[test]
    fn mj_button_inner_padding_collapsing_width_is_error() {
        let t = template_with_button(MjButton {
            content: "Go".into(),
            href: "https://example.com".into(),
            width: Some("200px".into()),
            inner_padding: Some("100px 100px".into()),
            ..Default::default()
        });
        let r = validate_template(&t);
        assert!(report_has(&r, "collapses the button"));
    }

    #[test]
    fn mj_button_width_too_small_for_brand_inner_padding_is_error() {
        let t = template_with_button(MjButton {
            content: "Go".into(),
            href: "https://example.com".into(),
            width: Some("40px".into()),
            ..Default::default()
        });
        let r = validate_template(&t);
        assert!(report_has(&r, "collapses the button"));
    }

    #[test]
    fn mj_button_inner_padding_fits_width() {
        let t = template_with_button(MjButton {
            content: "Go".into(),
            href: "https://example.com".into(),
            width: Some("200px".into()),
            inner_padding: Some("8px 16px".into()),
            border: Some("1px solid #ff0000".into()),
            ..Default::default()
        });
        let r = validate_template(&t);
        assert!(r.ok(), "{:?}", r.errors);
    }

    #[test]
    fn unknown_border_sides_is_error() {
        let t = template_with_button(MjButton {
            content: "Go".into(),
            href: "https://example.com".into(),
            border: Some("1px solid #000".into()),
            border_sides: Some("top,foo".into()),
            ..Default::default()
        });
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-button.border_sides"), "{:?}", r.errors);
    }

    #[test]
    fn mj_button_negative_inner_padding_is_error() {
        let t = template_with_button(MjButton {
            content: "Go".into(),
            href: "https://example.com".into(),
            inner_padding: Some("-2px".into()),
            ..Default::default()
        });
        let r = validate_template(&t);
        assert!(report_has(&r, "mj-button.inner_padding"));
    }
}
