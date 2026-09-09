//! Insert-picker kinds and the legal-target table from the design doc.
use crate::model::{
    Align, BodyNode, ColumnChild, EmailArticle, EmailCta, EmailFooter, EmailHeader, EmailHero,
    ImagePosition, MjAccordion, MjAccordionElement, MjButton, MjCarousel, MjCarouselImage,
    MjColumn, MjDivider, MjGroup, MjHero, MjImage, MjNavbar, MjNavbarLink, MjSection, MjSocial,
    MjSpacer, MjTable, MjText, MjWrapper, SectionChild, SocialMode, Template,
};

use super::tree::{Step, TreeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComponentKind {
    EmailHeader,
    EmailHero,
    EmailCta,
    EmailArticle,
    EmailFooter,
    MjSection,
    MjWrapper,
    MjHero,
    MjGroup,
    MjColumn,
    MjText,
    MjButton,
    MjImage,
    MjDivider,
    MjSpacer,
    MjSocial,
    MjTable,
    MjNavbar,
    MjNavbarLink,
    MjAccordion,
    MjAccordionElement,
    MjCarousel,
    MjCarouselImage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KindGroup {
    Block,
    Primitive,
}

impl ComponentKind {
    pub(super) fn all() -> &'static [Self] {
        &[
            Self::EmailHeader,
            Self::EmailHero,
            Self::EmailCta,
            Self::EmailArticle,
            Self::EmailFooter,
            Self::MjSection,
            Self::MjWrapper,
            Self::MjHero,
            Self::MjGroup,
            Self::MjColumn,
            Self::MjText,
            Self::MjButton,
            Self::MjImage,
            Self::MjDivider,
            Self::MjSpacer,
            Self::MjSocial,
            Self::MjTable,
            Self::MjNavbar,
            Self::MjNavbarLink,
            Self::MjAccordion,
            Self::MjAccordionElement,
            Self::MjCarousel,
            Self::MjCarouselImage,
        ]
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::EmailHeader => "email-header",
            Self::EmailHero => "email-hero",
            Self::EmailCta => "email-cta",
            Self::EmailArticle => "email-article",
            Self::EmailFooter => "email-footer",
            Self::MjSection => "mj-section",
            Self::MjWrapper => "mj-wrapper",
            Self::MjHero => "mj-hero",
            Self::MjGroup => "mj-group",
            Self::MjColumn => "mj-column",
            Self::MjText => "mj-text",
            Self::MjButton => "mj-button",
            Self::MjImage => "mj-image",
            Self::MjDivider => "mj-divider",
            Self::MjSpacer => "mj-spacer",
            Self::MjSocial => "mj-social",
            Self::MjTable => "mj-table",
            Self::MjNavbar => "mj-navbar",
            Self::MjNavbarLink => "mj-navbar-link",
            Self::MjAccordion => "mj-accordion",
            Self::MjAccordionElement => "mj-accordion-element",
            Self::MjCarousel => "mj-carousel",
            Self::MjCarouselImage => "mj-carousel-image",
        }
    }

    pub(super) fn group(self) -> KindGroup {
        match self {
            Self::EmailHeader
            | Self::EmailHero
            | Self::EmailCta
            | Self::EmailArticle
            | Self::EmailFooter => KindGroup::Block,
            _ => KindGroup::Primitive,
        }
    }

    pub(super) fn is_block(self) -> bool {
        matches!(self.group(), KindGroup::Block)
    }

    pub(super) fn is_layout(self) -> bool {
        matches!(self, Self::MjSection | Self::MjWrapper | Self::MjHero)
    }

    pub(super) fn is_leaf(self) -> bool {
        matches!(
            self,
            Self::MjText
                | Self::MjButton
                | Self::MjImage
                | Self::MjDivider
                | Self::MjSpacer
                | Self::MjSocial
                | Self::MjTable
                | Self::MjNavbar
                | Self::MjAccordion
                | Self::MjCarousel
        )
    }

    pub(super) fn is_nested(self) -> bool {
        matches!(
            self,
            Self::MjNavbarLink | Self::MjAccordionElement | Self::MjCarouselImage
        )
    }

    pub(super) fn legal_for(self, class: SelectionClass) -> bool {
        match class {
            SelectionClass::HeadBrand => false,
            SelectionClass::Body | SelectionClass::EmailBlock => {
                self.is_block() || self.is_layout() || self.is_leaf()
            }
            SelectionClass::Section { in_wrapper } => {
                if self.is_nested() {
                    false
                } else if self.is_block() || self == Self::MjWrapper {
                    !in_wrapper
                } else {
                    self.is_layout()
                        || matches!(self, Self::MjGroup | Self::MjColumn)
                        || self.is_leaf()
                }
            }
            SelectionClass::Wrapper => matches!(self, Self::MjSection | Self::MjHero),
            SelectionClass::Hero => self.is_leaf(),
            SelectionClass::Group => self == Self::MjColumn,
            SelectionClass::Column => self == Self::MjColumn || self.is_leaf(),
            SelectionClass::Leaf => self.is_leaf(),
            SelectionClass::Navbar | SelectionClass::NavbarLink => self == Self::MjNavbarLink,
            SelectionClass::Accordion | SelectionClass::AccordionEl => {
                self == Self::MjAccordionElement
            }
            SelectionClass::Carousel | SelectionClass::CarouselImg => self == Self::MjCarouselImage,
        }
    }

    pub(super) fn as_body_node(self) -> Option<BodyNode> {
        Some(match self {
            Self::EmailHeader => BodyNode::EmailHeader(EmailHeader {
                logo_src: String::new(),
                logo_alt: String::new(),
                logo_href: None,
                logo_width: "160px".into(),
                background_color: None,
            }),
            Self::EmailHero => BodyNode::EmailHero(EmailHero {
                image_src: String::new(),
                image_alt: String::new(),
                heading: "Heading".into(),
                subheading: String::new(),
                background_color: None,
            }),
            Self::EmailCta => BodyNode::EmailCta(EmailCta {
                heading: "Heading".into(),
                copy: String::new(),
                button_label: "Read more".into(),
                button_href: "https://example.com".into(),
                background_color: None,
            }),
            Self::EmailArticle => BodyNode::EmailArticle(EmailArticle {
                image_src: String::new(),
                image_alt: String::new(),
                title: "Title".into(),
                copy: String::new(),
                link_label: String::new(),
                link_href: String::new(),
                image_position: ImagePosition::Top,
            }),
            Self::EmailFooter => BodyNode::EmailFooter(EmailFooter {
                company_name: String::new(),
                address_lines: vec!["123 Main St".into()],
                unsubscribe_label: "Unsubscribe".into(),
                unsubscribe_href: "*|UNSUB|*".into(),
                social: Vec::new(),
                copyright: None,
            }),
            Self::MjSection => BodyNode::MjSection(empty_section()),
            Self::MjWrapper => BodyNode::MjWrapper(MjWrapper {
                background_color: None,
                padding: None,
                full_width: false,
                children: Vec::new(),
                ..Default::default()
            }),
            Self::MjHero => BodyNode::MjHero(empty_hero()),
            _ => return None,
        })
    }

    pub(super) fn as_leaf(self) -> Option<ColumnChild> {
        Some(match self {
            Self::MjText => ColumnChild::MjText(MjText {
                content: "Write something.".into(),
                align: None,
                font_size: None,
                font_family: None,
                color: None,
                padding: None,
                ..Default::default()
            }),
            Self::MjButton => ColumnChild::MjButton(MjButton {
                content: "Read more".into(),
                href: "https://example.com".into(),
                background_color: None,
                color: None,
                align: Some(Align::Center),
                font_family: None,
                border_radius: None,
                width: None,
                padding: None,
                ..Default::default()
            }),
            Self::MjImage => ColumnChild::MjImage(MjImage {
                src: "https://dummyimage.com/600x200/cccccc/000000".into(),
                alt: "Image".into(),
                href: None,
                width: None,
                align: None,
                fluid_on_mobile: true,
                padding: None,
                ..Default::default()
            }),
            Self::MjDivider => ColumnChild::MjDivider(MjDivider {
                border_color: None,
                border_width: None,
                padding: None,
                ..Default::default()
            }),
            Self::MjSpacer => ColumnChild::MjSpacer(MjSpacer {
                height: "24px".into(),
                ..Default::default()
            }),
            Self::MjSocial => ColumnChild::MjSocial(MjSocial {
                mode: SocialMode::Horizontal,
                align: None,
                icon_size: "32px".into(),
                elements: Vec::new(),
                ..Default::default()
            }),
            Self::MjTable => ColumnChild::MjTable(MjTable {
                content: "<tr><td></td></tr>".into(),
                font_size: None,
                color: None,
                padding: None,
                ..Default::default()
            }),
            Self::MjNavbar => ColumnChild::MjNavbar(empty_navbar()),
            Self::MjAccordion => ColumnChild::MjAccordion(empty_accordion()),
            Self::MjCarousel => ColumnChild::MjCarousel(empty_carousel()),
            _ => return None,
        })
    }

    pub(super) fn as_navbar_link(self) -> Option<MjNavbarLink> {
        (self == Self::MjNavbarLink).then(empty_navbar_link)
    }

    pub(super) fn as_accordion_element(self) -> Option<MjAccordionElement> {
        (self == Self::MjAccordionElement).then(empty_accordion_element)
    }

    pub(super) fn as_carousel_image(self) -> Option<MjCarouselImage> {
        (self == Self::MjCarouselImage).then(empty_carousel_image)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectionClass {
    HeadBrand,
    Body,
    EmailBlock,
    Section { in_wrapper: bool },
    Wrapper,
    Hero,
    Group,
    Column,
    Leaf,
    Navbar,
    NavbarLink,
    Accordion,
    AccordionEl,
    Carousel,
    CarouselImg,
}

pub(super) fn classify(template: Option<&Template>, id: &TreeId) -> SelectionClass {
    match id {
        TreeId::Head | TreeId::Brand => SelectionClass::HeadBrand,
        TreeId::Body => SelectionClass::Body,
        TreeId::Path(path) => classify_path(template, path),
    }
}

fn classify_path(template: Option<&Template>, path: &[Step]) -> SelectionClass {
    let Some(last) = path.last() else {
        return SelectionClass::Body;
    };
    match last {
        Step::NavbarLink(_) => SelectionClass::NavbarLink,
        Step::AccordionEl(_) => SelectionClass::AccordionEl,
        Step::CarouselImg(_) => SelectionClass::CarouselImg,
        Step::ColComp(_) | Step::HeroChild(_) => classify_column_child(template, path),
        Step::GroupCol(_) => SelectionClass::Column,
        Step::SectionChild(_) => {
            let Some(t) = template else {
                return SelectionClass::Column;
            };
            match locate_section_child(t, path) {
                Some(SectionChild::MjGroup(_)) => SelectionClass::Group,
                _ => SelectionClass::Column,
            }
        }
        Step::BodyNode(_) | Step::WrapperChild(_) => {
            let in_wrapper = path.iter().any(|s| matches!(s, Step::WrapperChild(_)));
            match body_node_at(template, path) {
                Some(BodyNode::MjSection(_)) => SelectionClass::Section { in_wrapper },
                Some(BodyNode::MjWrapper(_)) => SelectionClass::Wrapper,
                Some(BodyNode::MjHero(_)) => SelectionClass::Hero,
                Some(
                    BodyNode::EmailHeader(_)
                    | BodyNode::EmailHero(_)
                    | BodyNode::EmailCta(_)
                    | BodyNode::EmailArticle(_)
                    | BodyNode::EmailFooter(_),
                ) => SelectionClass::EmailBlock,
                None => SelectionClass::Body,
            }
        }
    }
}

fn classify_column_child(template: Option<&Template>, path: &[Step]) -> SelectionClass {
    match column_child_at(template, path) {
        Some(ColumnChild::MjNavbar(_)) => SelectionClass::Navbar,
        Some(ColumnChild::MjAccordion(_)) => SelectionClass::Accordion,
        Some(ColumnChild::MjCarousel(_)) => SelectionClass::Carousel,
        _ => SelectionClass::Leaf,
    }
}

fn column_child_at<'a>(template: Option<&'a Template>, path: &[Step]) -> Option<&'a ColumnChild> {
    let t = template?;
    let mut steps = path.iter();
    let Step::BodyNode(i) = steps.next()? else {
        return None;
    };
    let mut node = t.body.nodes.get(*i)?;
    loop {
        match steps.next() {
            Some(Step::WrapperChild(j)) => {
                let BodyNode::MjWrapper(w) = node else {
                    return None;
                };
                node = w.children.get(*j)?;
            }
            Some(Step::SectionChild(j)) => {
                let BodyNode::MjSection(s) = node else {
                    return None;
                };
                return match s.children.get(*j)? {
                    SectionChild::MjColumn(c) => take_col_comp(c, steps),
                    SectionChild::MjGroup(g) => {
                        let Step::GroupCol(k) = steps.next()? else {
                            return None;
                        };
                        take_col_comp(g.children.get(*k)?, steps)
                    }
                };
            }
            Some(Step::HeroChild(j)) => {
                let BodyNode::MjHero(h) = node else {
                    return None;
                };
                return h.children.get(*j);
            }
            _ => return None,
        }
    }
}

fn take_col_comp<'a>(
    c: &'a MjColumn,
    mut steps: std::slice::Iter<'_, Step>,
) -> Option<&'a ColumnChild> {
    let Step::ColComp(i) = steps.next()? else {
        return None;
    };
    c.components.get(*i)
}

fn body_node_at<'a>(template: Option<&'a Template>, path: &[Step]) -> Option<&'a BodyNode> {
    let t = template?;
    let mut node: Option<&BodyNode> = None;
    for step in path {
        match step {
            Step::BodyNode(i) => node = t.body.nodes.get(*i),
            Step::WrapperChild(i) => {
                let BodyNode::MjWrapper(w) = node? else {
                    return None;
                };
                node = w.children.get(*i);
            }
            _ => break,
        }
    }
    node
}

fn locate_section_child<'a>(t: &'a Template, path: &[Step]) -> Option<&'a SectionChild> {
    let (last, parent) = path.split_last()?;
    let Step::SectionChild(i) = last else {
        return None;
    };
    match body_node_at(Some(t), parent)? {
        BodyNode::MjSection(s) => s.children.get(*i),
        _ => None,
    }
}

pub(super) fn empty_column() -> MjColumn {
    MjColumn::default()
}

pub(super) fn empty_section() -> MjSection {
    MjSection::default()
}

pub(super) fn empty_hero() -> MjHero {
    MjHero::default()
}

pub(super) fn empty_navbar() -> MjNavbar {
    MjNavbar::default()
}

pub(super) fn empty_navbar_link() -> MjNavbarLink {
    MjNavbarLink {
        href: "https://example.com".into(),
        content: "Link".into(),
        ..Default::default()
    }
}

pub(super) fn empty_accordion() -> MjAccordion {
    MjAccordion::default()
}

pub(super) fn empty_accordion_element() -> MjAccordionElement {
    MjAccordionElement {
        title: "Title".into(),
        content: "Write something.".into(),
        ..Default::default()
    }
}

pub(super) fn empty_carousel() -> MjCarousel {
    MjCarousel::default()
}

pub(super) fn empty_carousel_image() -> MjCarouselImage {
    MjCarouselImage {
        src: "https://dummyimage.com/600x200/cccccc/000000".into(),
        alt: "Image".into(),
        ..Default::default()
    }
}

pub(super) fn empty_group() -> MjGroup {
    MjGroup {
        children: vec![empty_column()],
        ..Default::default()
    }
}

pub(super) fn wrap_leaf_in_section(leaf: ColumnChild) -> BodyNode {
    let mut col = empty_column();
    col.width = Some("100%".into());
    col.components.push(leaf);
    BodyNode::MjSection(MjSection {
        background_color: None,
        padding: None,
        full_width: false,
        children: vec![SectionChild::MjColumn(col)],
        ..Default::default()
    })
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PickerRow {
    Header(&'static str),
    Kind(ComponentKind),
}

pub(super) fn picker_rows(class: SelectionClass, query: &str) -> Vec<PickerRow> {
    let needle = query.trim().to_lowercase();
    let mut blocks = Vec::new();
    let mut prims = Vec::new();
    for &kind in ComponentKind::all() {
        if !kind.legal_for(class) {
            continue;
        }
        if !needle.is_empty() && !kind.label().contains(&needle) {
            continue;
        }
        match kind.group() {
            KindGroup::Block => blocks.push(kind),
            KindGroup::Primitive => prims.push(kind),
        }
    }
    let mut rows = Vec::new();
    if !blocks.is_empty() {
        rows.push(PickerRow::Header("-- blocks --"));
        rows.extend(blocks.into_iter().map(PickerRow::Kind));
    }
    if !prims.is_empty() {
        rows.push(PickerRow::Header("-- primitives --"));
        rows.extend(prims.into_iter().map(PickerRow::Kind));
    }
    rows
}

pub(super) fn first_kind_index(rows: &[PickerRow]) -> usize {
    rows.iter()
        .position(|r| matches!(r, PickerRow::Kind(_)))
        .unwrap_or(0)
}

pub(super) fn move_picker_selection(rows: &[PickerRow], selected: usize, delta: isize) -> usize {
    if rows.is_empty() {
        return 0;
    }
    let mut i = selected.min(rows.len() - 1);
    let len = rows.len() as isize;
    for _ in 0..rows.len() {
        let next = (i as isize + delta).rem_euclid(len) as usize;
        i = next;
        if matches!(rows[i], PickerRow::Kind(_)) {
            return i;
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_labels(class: SelectionClass) -> Vec<&'static str> {
        picker_rows(class, "")
            .into_iter()
            .filter_map(|r| match r {
                PickerRow::Kind(k) => Some(k.label()),
                PickerRow::Header(_) => None,
            })
            .collect()
    }

    #[test]
    fn column_picker_includes_navbar_not_navbar_link() {
        let labels = kind_labels(SelectionClass::Column);
        assert!(labels.contains(&"mj-navbar"));
        assert!(labels.contains(&"mj-accordion"));
        assert!(labels.contains(&"mj-carousel"));
        assert!(labels.contains(&"mj-text"));
        assert!(!labels.contains(&"mj-navbar-link"));
        assert!(!labels.contains(&"mj-accordion-element"));
        assert!(!labels.contains(&"mj-carousel-image"));
        assert!(!labels.contains(&"email-header"));
        assert!(!labels.contains(&"mj-section"));
    }

    #[test]
    fn section_picker_excludes_nested_kinds() {
        let labels = kind_labels(SelectionClass::Section { in_wrapper: false });
        assert!(labels.contains(&"mj-column"));
        assert!(labels.contains(&"mj-group"));
        assert!(labels.contains(&"mj-navbar"));
        assert!(!labels.contains(&"mj-navbar-link"));
        assert!(!labels.contains(&"mj-accordion-element"));
        assert!(!labels.contains(&"mj-carousel-image"));
    }

    #[test]
    fn wrapper_picker_is_section_and_hero_only() {
        let labels = kind_labels(SelectionClass::Wrapper);
        assert_eq!(labels, vec!["mj-section", "mj-hero"]);
    }

    #[test]
    fn navbar_picker_is_navbar_link_only() {
        let labels = kind_labels(SelectionClass::Navbar);
        assert_eq!(labels, vec!["mj-navbar-link"]);
    }

    #[test]
    fn group_picker_is_column_only() {
        let labels = kind_labels(SelectionClass::Group);
        assert_eq!(labels, vec!["mj-column"]);
    }

    #[test]
    fn mj_table_insert_default_is_row_markup() {
        let Some(ColumnChild::MjTable(t)) = ComponentKind::MjTable.as_leaf() else {
            panic!("expected mj-table leaf");
        };
        assert_eq!(t.content, "<tr><td></td></tr>");
        assert!(!t.content.to_ascii_lowercase().contains("<table"));
    }
}
