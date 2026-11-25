//! Structure outline. Labels come from the template model.

use dd_emailforge::model::{
    BodyNode, ColumnChild, MjAccordion, MjCarousel, MjColumn, MjGroup, MjHero, MjNavbar,
    MjSection, MjSocial, MjWrapper, SectionChild, Template,
};

pub fn outline_lines(template: &Template) -> Vec<String> {
    let mut lines = vec!["HEAD".to_string(), "BRAND".to_string(), "BODY".to_string()];
    for node in &template.body.nodes {
        push_body(&mut lines, node, 1);
    }
    lines
}

fn push_body(lines: &mut Vec<String>, node: &BodyNode, depth: usize) {
    match node {
        BodyNode::MjSection(section) => push_section(lines, section, depth),
        BodyNode::MjWrapper(wrapper) => push_wrapper(lines, wrapper, depth),
        BodyNode::MjHero(hero) => push_hero(lines, hero, depth),
        BodyNode::EmailHeader(_) => push(lines, depth, "email-header"),
        BodyNode::EmailHero(_) => push(lines, depth, "email-hero"),
        BodyNode::EmailCta(_) => push(lines, depth, "email-cta"),
        BodyNode::EmailArticle(_) => push(lines, depth, "email-article"),
        BodyNode::EmailFooter(_) => push(lines, depth, "email-footer"),
    }
}

fn push_section(lines: &mut Vec<String>, section: &MjSection, depth: usize) {
    push(
        lines,
        depth,
        &named("section", section.label.as_deref()),
    );
    for child in &section.children {
        match child {
            SectionChild::MjColumn(column) => push_column(lines, column, depth + 1),
            SectionChild::MjGroup(group) => push_group(lines, group, depth + 1),
        }
    }
}

fn push_wrapper(lines: &mut Vec<String>, wrapper: &MjWrapper, depth: usize) {
    push(
        lines,
        depth,
        &named("wrapper", wrapper.label.as_deref()),
    );
    for child in &wrapper.children {
        push_body(lines, child, depth + 1);
    }
}

fn push_hero(lines: &mut Vec<String>, hero: &MjHero, depth: usize) {
    push(lines, depth, "hero");
    for child in &hero.children {
        push_component(lines, child, depth + 1);
    }
}

fn push_group(lines: &mut Vec<String>, group: &MjGroup, depth: usize) {
    push(lines, depth, "group");
    for column in &group.children {
        push_column(lines, column, depth + 1);
    }
}

fn push_column(lines: &mut Vec<String>, column: &MjColumn, depth: usize) {
    let label = match column
        .width
        .as_deref()
        .map(str::trim)
        .filter(|width| !width.is_empty())
    {
        Some(width) => format!("column {width}"),
        None => "column".to_string(),
    };
    push(lines, depth, &label);
    for child in &column.components {
        push_component(lines, child, depth + 1);
    }
}

fn push_component(lines: &mut Vec<String>, child: &ColumnChild, depth: usize) {
    match child {
        ColumnChild::MjText(_) => push(lines, depth, "mj-text"),
        ColumnChild::MjButton(_) => push(lines, depth, "mj-button"),
        ColumnChild::MjImage(_) => push(lines, depth, "mj-image"),
        ColumnChild::MjDivider(_) => push(lines, depth, "mj-divider"),
        ColumnChild::MjSpacer(_) => push(lines, depth, "mj-spacer"),
        ColumnChild::MjSocial(social) => push_social(lines, social, depth),
        ColumnChild::MjTable(_) => push(lines, depth, "mj-table"),
        ColumnChild::MjNavbar(navbar) => push_navbar(lines, navbar, depth),
        ColumnChild::MjAccordion(accordion) => push_accordion(lines, accordion, depth),
        ColumnChild::MjCarousel(carousel) => push_carousel(lines, carousel, depth),
    }
}

fn push_social(lines: &mut Vec<String>, social: &MjSocial, depth: usize) {
    push(lines, depth, "mj-social");
    for element in &social.elements {
        push(
            lines,
            depth + 1,
            &format!("social {:?}", element.name),
        );
    }
}

fn push_navbar(lines: &mut Vec<String>, navbar: &MjNavbar, depth: usize) {
    push(lines, depth, "mj-navbar");
    for link in &navbar.links {
        let label = if link.content.trim().is_empty() {
            "navbar-link".to_string()
        } else {
            format!("navbar-link {}", link.content.trim())
        };
        push(lines, depth + 1, &label);
    }
}

fn push_accordion(lines: &mut Vec<String>, accordion: &MjAccordion, depth: usize) {
    push(lines, depth, "mj-accordion");
    for element in &accordion.elements {
        let label = if element.title.trim().is_empty() {
            "accordion-element".to_string()
        } else {
            format!("accordion-element {}", element.title.trim())
        };
        push(lines, depth + 1, &label);
    }
}

fn push_carousel(lines: &mut Vec<String>, carousel: &MjCarousel, depth: usize) {
    push(lines, depth, "mj-carousel");
    for image in &carousel.images {
        let label = if image.alt.trim().is_empty() {
            "carousel-image".to_string()
        } else {
            format!("carousel-image {}", image.alt.trim())
        };
        push(lines, depth + 1, &label);
    }
}

fn named(kind: &str, label: Option<&str>) -> String {
    match label.map(str::trim).filter(|label| !label.is_empty()) {
        Some(label) => format!("{kind} {label}"),
        None => kind.to_string(),
    }
}

fn push(lines: &mut Vec<String>, depth: usize, label: &str) {
    lines.push(format!("{}{label}", "  ".repeat(depth)));
}
