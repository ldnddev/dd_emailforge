//! Structure tree: identity, build, expand, keyboard nav.
use std::collections::HashSet;

use super::App;
use super::toasts::ToastLevel;
use crate::model::{
    BodyNode, ColumnChild, MjAccordion, MjCarousel, MjColumn, MjGroup, MjHero, MjNavbar, MjSection,
    SectionChild, Template,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TreeId {
    Head,
    Brand,
    Body,
    Path(Vec<Step>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    BodyNode(usize),
    WrapperChild(usize),
    SectionChild(usize),
    GroupCol(usize),
    ColComp(usize),
    HeroChild(usize),
    NavbarLink(usize),
    AccordionEl(usize),
    CarouselImg(usize),
}

#[derive(Clone, Debug)]
pub struct TreeRow {
    pub id: TreeId,
    pub label: String,
    pub prefix: String,
    pub expandable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneFocus {
    Structure,
    Details,
}

pub fn master_detail_tree_width(area_width: u16) -> Option<u16> {
    if area_width < 48 {
        return None;
    }
    let preferred = ((area_width as f32) * 0.32) as u16;
    Some(preferred.max(24).min(area_width.saturating_sub(20)))
}

pub fn build_tree(template: Option<&Template>, collapsed: &HashSet<TreeId>) -> Vec<TreeRow> {
    let mut rows = Vec::new();
    rows.push(TreeRow {
        id: TreeId::Head,
        label: "[HEAD] mj-head".into(),
        prefix: String::new(),
        expandable: false,
    });
    rows.push(TreeRow {
        id: TreeId::Brand,
        label: "[BRAND] brand".into(),
        prefix: String::new(),
        expandable: false,
    });
    let Some(t) = template else {
        rows.push(TreeRow {
            id: TreeId::Body,
            label: "[BODY] mj-body".into(),
            prefix: String::new(),
            expandable: false,
        });
        return rows;
    };
    let body_expandable = !t.body.nodes.is_empty();
    let body_expanded = body_expandable && !collapsed.contains(&TreeId::Body);
    rows.push(TreeRow {
        id: TreeId::Body,
        label: format!("[BODY] mj-body ({})", t.body.nodes.len()),
        prefix: String::new(),
        expandable: body_expandable,
    });
    if body_expanded {
        let n = t.body.nodes.len();
        for (i, node) in t.body.nodes.iter().enumerate() {
            push_body_node(
                &mut rows,
                node,
                vec![Step::BodyNode(i)],
                &[],
                i + 1 == n,
                i + 1,
                collapsed,
            );
        }
    }
    rows
}

fn push_body_node(
    rows: &mut Vec<TreeRow>,
    node: &BodyNode,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    number: usize,
    collapsed: &HashSet<TreeId>,
) {
    let id = TreeId::Path(path.clone());
    match node {
        BodyNode::MjSection(s) => {
            let expandable = !s.children.is_empty();
            let expanded = expandable && !collapsed.contains(&id);
            rows.push(row(
                id.clone(),
                numbered_kind(number, "mj-section", s.label.as_deref()),
                ancestors_last,
                is_last,
                expandable,
            ));
            if expanded {
                let mut next = ancestors_last.to_vec();
                next.push(is_last);
                push_section_children(rows, s, &path, &next, collapsed);
            }
        }
        BodyNode::MjWrapper(w) => {
            let expandable = !w.children.is_empty();
            let expanded = expandable && !collapsed.contains(&id);
            rows.push(row(
                id.clone(),
                numbered_kind(number, "mj-wrapper", w.label.as_deref()),
                ancestors_last,
                is_last,
                expandable,
            ));
            if expanded {
                let mut next = ancestors_last.to_vec();
                next.push(is_last);
                let n = w.children.len();
                for (i, child) in w.children.iter().enumerate() {
                    let mut p = path.clone();
                    p.push(Step::WrapperChild(i));
                    push_body_node(rows, child, p, &next, i + 1 == n, i + 1, collapsed);
                }
            }
        }
        BodyNode::MjHero(h) => {
            let expandable = !h.children.is_empty();
            let expanded = expandable && !collapsed.contains(&id);
            rows.push(row(
                id.clone(),
                format!("{number}. mj-hero"),
                ancestors_last,
                is_last,
                expandable,
            ));
            if expanded {
                let mut next = ancestors_last.to_vec();
                next.push(is_last);
                push_hero_children(rows, h, &path, &next, collapsed);
            }
        }
        BodyNode::EmailHeader(_) => {
            rows.push(row(
                id,
                format!("{number}. email-header"),
                ancestors_last,
                is_last,
                false,
            ));
        }
        BodyNode::EmailHero(_) => {
            rows.push(row(
                id,
                format!("{number}. email-hero"),
                ancestors_last,
                is_last,
                false,
            ));
        }
        BodyNode::EmailCta(_) => {
            rows.push(row(
                id,
                format!("{number}. email-cta"),
                ancestors_last,
                is_last,
                false,
            ));
        }
        BodyNode::EmailArticle(_) => {
            rows.push(row(
                id,
                format!("{number}. email-article"),
                ancestors_last,
                is_last,
                false,
            ));
        }
        BodyNode::EmailFooter(_) => {
            rows.push(row(
                id,
                format!("{number}. email-footer"),
                ancestors_last,
                is_last,
                false,
            ));
        }
    }
}

fn push_section_children(
    rows: &mut Vec<TreeRow>,
    s: &MjSection,
    path: &[Step],
    ancestors_last: &[bool],
    collapsed: &HashSet<TreeId>,
) {
    let n = s.children.len();
    for (i, child) in s.children.iter().enumerate() {
        let mut p = path.to_vec();
        p.push(Step::SectionChild(i));
        let last = i + 1 == n;
        match child {
            SectionChild::MjColumn(c) => {
                push_column(rows, c, p, ancestors_last, last, collapsed);
            }
            SectionChild::MjGroup(g) => {
                push_group(rows, g, p, ancestors_last, last, collapsed);
            }
        }
    }
}

fn push_group(
    rows: &mut Vec<TreeRow>,
    g: &MjGroup,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    collapsed: &HashSet<TreeId>,
) {
    let id = TreeId::Path(path.clone());
    let expandable = !g.children.is_empty();
    let expanded = expandable && !collapsed.contains(&id);
    rows.push(row(
        id.clone(),
        "mj-group".into(),
        ancestors_last,
        is_last,
        expandable,
    ));
    if expanded {
        let mut next = ancestors_last.to_vec();
        next.push(is_last);
        let n = g.children.len();
        for (i, col) in g.children.iter().enumerate() {
            let mut p = path.clone();
            p.push(Step::GroupCol(i));
            push_column(rows, col, p, &next, i + 1 == n, collapsed);
        }
    }
}

fn push_column(
    rows: &mut Vec<TreeRow>,
    c: &MjColumn,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    collapsed: &HashSet<TreeId>,
) {
    let id = TreeId::Path(path.clone());
    let expandable = !c.components.is_empty();
    let expanded = expandable && !collapsed.contains(&id);
    let label = match c.width.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(w) => format!("mj-column {w}"),
        None => "mj-column".into(),
    };
    rows.push(row(id.clone(), label, ancestors_last, is_last, expandable));
    if expanded {
        let mut next = ancestors_last.to_vec();
        next.push(is_last);
        let n = c.components.len();
        for (i, comp) in c.components.iter().enumerate() {
            let mut p = path.clone();
            p.push(Step::ColComp(i));
            push_column_child(rows, comp, p, &next, i + 1 == n, collapsed);
        }
    }
}

fn push_hero_children(
    rows: &mut Vec<TreeRow>,
    h: &MjHero,
    path: &[Step],
    ancestors_last: &[bool],
    collapsed: &HashSet<TreeId>,
) {
    let n = h.children.len();
    for (i, comp) in h.children.iter().enumerate() {
        let mut p = path.to_vec();
        p.push(Step::HeroChild(i));
        push_column_child(rows, comp, p, ancestors_last, i + 1 == n, collapsed);
    }
}

fn push_column_child(
    rows: &mut Vec<TreeRow>,
    c: &ColumnChild,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    collapsed: &HashSet<TreeId>,
) {
    match c {
        ColumnChild::MjNavbar(n) => push_navbar(rows, n, path, ancestors_last, is_last, collapsed),
        ColumnChild::MjAccordion(a) => {
            push_accordion(rows, a, path, ancestors_last, is_last, collapsed)
        }
        ColumnChild::MjCarousel(car) => {
            push_carousel(rows, car, path, ancestors_last, is_last, collapsed)
        }
        other => rows.push(row(
            TreeId::Path(path),
            column_child_label(other),
            ancestors_last,
            is_last,
            false,
        )),
    }
}

fn push_navbar(
    rows: &mut Vec<TreeRow>,
    n: &MjNavbar,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    collapsed: &HashSet<TreeId>,
) {
    let id = TreeId::Path(path.clone());
    let expandable = !n.links.is_empty();
    let expanded = expandable && !collapsed.contains(&id);
    rows.push(row(
        id,
        "mj-navbar".into(),
        ancestors_last,
        is_last,
        expandable,
    ));
    if expanded {
        let mut next = ancestors_last.to_vec();
        next.push(is_last);
        let count = n.links.len();
        for (i, _) in n.links.iter().enumerate() {
            let mut p = path.clone();
            p.push(Step::NavbarLink(i));
            rows.push(row(
                TreeId::Path(p),
                "mj-navbar-link".into(),
                &next,
                i + 1 == count,
                false,
            ));
        }
    }
}

fn push_accordion(
    rows: &mut Vec<TreeRow>,
    a: &MjAccordion,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    collapsed: &HashSet<TreeId>,
) {
    let id = TreeId::Path(path.clone());
    let expandable = !a.elements.is_empty();
    let expanded = expandable && !collapsed.contains(&id);
    rows.push(row(
        id,
        "mj-accordion".into(),
        ancestors_last,
        is_last,
        expandable,
    ));
    if expanded {
        let mut next = ancestors_last.to_vec();
        next.push(is_last);
        let count = a.elements.len();
        for (i, _) in a.elements.iter().enumerate() {
            let mut p = path.clone();
            p.push(Step::AccordionEl(i));
            rows.push(row(
                TreeId::Path(p),
                "mj-accordion-element".into(),
                &next,
                i + 1 == count,
                false,
            ));
        }
    }
}

fn push_carousel(
    rows: &mut Vec<TreeRow>,
    c: &MjCarousel,
    path: Vec<Step>,
    ancestors_last: &[bool],
    is_last: bool,
    collapsed: &HashSet<TreeId>,
) {
    let id = TreeId::Path(path.clone());
    let expandable = !c.images.is_empty();
    let expanded = expandable && !collapsed.contains(&id);
    rows.push(row(
        id,
        "mj-carousel".into(),
        ancestors_last,
        is_last,
        expandable,
    ));
    if expanded {
        let mut next = ancestors_last.to_vec();
        next.push(is_last);
        let count = c.images.len();
        for (i, _) in c.images.iter().enumerate() {
            let mut p = path.clone();
            p.push(Step::CarouselImg(i));
            rows.push(row(
                TreeId::Path(p),
                "mj-carousel-image".into(),
                &next,
                i + 1 == count,
                false,
            ));
        }
    }
}

fn column_child_label(c: &ColumnChild) -> String {
    match c {
        ColumnChild::MjText(_) => "mj-text".into(),
        ColumnChild::MjButton(_) => "mj-button".into(),
        ColumnChild::MjImage(_) => "mj-image".into(),
        ColumnChild::MjDivider(_) => "mj-divider".into(),
        ColumnChild::MjSpacer(_) => "mj-spacer".into(),
        ColumnChild::MjSocial(_) => "mj-social".into(),
        ColumnChild::MjTable(_) => "mj-table".into(),
        ColumnChild::MjNavbar(_) => "mj-navbar".into(),
        ColumnChild::MjAccordion(_) => "mj-accordion".into(),
        ColumnChild::MjCarousel(_) => "mj-carousel".into(),
    }
}

fn numbered_kind(number: usize, kind: &str, label: Option<&str>) -> String {
    match label.map(str::trim).filter(|s| !s.is_empty()) {
        Some(l) => format!("{number}. {kind}  {l}"),
        None => format!("{number}. {kind}"),
    }
}

fn row(
    id: TreeId,
    label: String,
    ancestors_last: &[bool],
    is_last: bool,
    expandable: bool,
) -> TreeRow {
    let mut prefix = String::new();
    for &last in ancestors_last {
        prefix.push_str(if last { "   " } else { "│  " });
    }
    prefix.push_str(if is_last { "└─ " } else { "├─ " });
    TreeRow {
        id,
        label,
        prefix,
        expandable,
    }
}

pub fn selected_label(rows: &[TreeRow], selected: usize) -> String {
    rows.get(selected)
        .map(|r| r.label.clone())
        .unwrap_or_else(|| "—".into())
}

fn parent_id(id: &TreeId) -> Option<TreeId> {
    match id {
        TreeId::Path(p) if p.len() == 1 => Some(TreeId::Body),
        TreeId::Path(p) => {
            let mut n = p.clone();
            n.pop();
            Some(TreeId::Path(n))
        }
        _ => None,
    }
}

impl App {
    pub(in crate::tui) fn tree_rows(&self) -> Vec<TreeRow> {
        build_tree(self.template.as_ref(), &self.collapsed)
    }

    pub(in crate::tui) fn clamp_tree_selection(&mut self) {
        let n = self.tree_rows().len();
        if n == 0 {
            self.selected_row = 0;
        } else if self.selected_row >= n {
            self.selected_row = n - 1;
        }
    }

    pub(in crate::tui) fn tree_move(&mut self, delta: isize) {
        let n = self.tree_rows().len() as isize;
        if n == 0 {
            return;
        }
        let next = (self.selected_row as isize + delta).clamp(0, n - 1);
        self.selected_row = next as usize;
    }

    pub(in crate::tui) fn tree_home(&mut self) {
        self.selected_row = 0;
    }

    pub(in crate::tui) fn tree_end(&mut self) {
        let n = self.tree_rows().len();
        self.selected_row = n.saturating_sub(1);
    }

    pub(in crate::tui) fn tree_expand(&mut self) {
        let rows = self.tree_rows();
        let Some(row) = rows.get(self.selected_row) else {
            return;
        };
        if row.expandable {
            self.collapsed.remove(&row.id);
        }
    }

    pub(in crate::tui) fn tree_collapse(&mut self) {
        let rows = self.tree_rows();
        let Some(row) = rows.get(self.selected_row) else {
            return;
        };
        if row.expandable && !self.collapsed.contains(&row.id) {
            self.collapsed.insert(row.id.clone());
            return;
        }
        if let Some(parent) = parent_id(&row.id) {
            if let Some(idx) = rows.iter().position(|r| r.id == parent) {
                self.selected_row = idx;
            }
        }
    }

    pub(in crate::tui) fn tree_toggle_expand(&mut self) {
        let rows = self.tree_rows();
        let Some(row) = rows.get(self.selected_row) else {
            return;
        };
        if !row.expandable {
            return;
        }
        if !self.collapsed.remove(&row.id) {
            self.collapsed.insert(row.id.clone());
        }
    }

    pub(in crate::tui) fn tree_enter(&mut self) {
        let Some(template) = self.template.as_ref() else {
            self.push_toast(
                ToastLevel::Warning,
                "No template open. Run: dd_emailforge init <dir>",
            );
            return;
        };
        let rows = self.tree_rows();
        let Some(row) = rows.get(self.selected_row) else {
            return;
        };
        let Some(state) = crate::tui::cursor::form_for(template, &row.id) else {
            self.push_toast(ToastLevel::Warning, "Nothing to edit here.");
            return;
        };
        let cursor_pos = state
            .form
            .fields
            .first()
            .map(|f| state.get(f.id).chars().count())
            .unwrap_or(0);
        self.modal = Some(super::Modal::FormEdit {
            cursor: row.id.clone(),
            state,
            cursor_pos,
            scroll_offset: 0,
            drill_stack: Vec::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BodyNode, ColumnChild, EmailHeader, MjColumn, MjSection, MjText, MjWrapper, SectionChild,
        Template,
    };

    #[test]
    fn width_none_below_48() {
        assert!(master_detail_tree_width(47).is_none());
        let w = master_detail_tree_width(80).unwrap();
        assert!(w >= 24);
        assert!(w <= 60);
    }

    #[test]
    fn empty_template_has_three_roots() {
        let rows = build_tree(None, &HashSet::new());
        assert_eq!(rows.len(), 3);
        assert!(matches!(rows[0].id, TreeId::Head));
        assert!(matches!(rows[1].id, TreeId::Brand));
        assert!(matches!(rows[2].id, TreeId::Body));
    }

    #[test]
    fn body_children_listed_when_expanded() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailHeader(EmailHeader {
            logo_src: String::new(),
            logo_alt: String::new(),
            logo_href: None,
            logo_width: "160px".into(),
            background_color: None,
        }));
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![SectionChild::MjColumn(MjColumn {
                width: Some("100%".into()),
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![ColumnChild::MjText(MjText {
                    content: "Hi".into(),
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
        let rows = build_tree(Some(&t), &HashSet::new());
        let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
        assert!(labels.contains(&"[HEAD] mj-head"));
        assert!(labels.contains(&"[BRAND] brand"));
        assert!(labels.iter().any(|l| l.starts_with("[BODY]")));
        assert!(labels.contains(&"1. email-header"));
        assert!(labels.contains(&"2. mj-section"));
        assert!(labels.contains(&"mj-column 100%"));
        assert!(labels.contains(&"mj-text"));
    }

    #[test]
    fn collapsed_body_hides_children() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::EmailHeader(EmailHeader {
            logo_src: String::new(),
            logo_alt: String::new(),
            logo_href: None,
            logo_width: "160px".into(),
            background_color: None,
        }));
        let mut collapsed = HashSet::new();
        collapsed.insert(TreeId::Body);
        let rows = build_tree(Some(&t), &collapsed);
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn section_and_wrapper_labels_show_in_tree() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            label: Some("hero".into()),
            ..Default::default()
        }));
        t.body.nodes.push(BodyNode::MjWrapper(MjWrapper {
            label: Some("footer".into()),
            ..Default::default()
        }));
        t.body.nodes.push(BodyNode::MjSection(MjSection::default()));
        let rows = build_tree(Some(&t), &HashSet::new());
        let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
        assert!(labels.contains(&"1. mj-section  hero"), "{labels:?}");
        assert!(labels.contains(&"2. mj-wrapper  footer"), "{labels:?}");
        assert!(labels.contains(&"3. mj-section"), "{labels:?}");
        assert!(!labels.contains(&"3. mj-section  "), "{labels:?}");
    }
}
