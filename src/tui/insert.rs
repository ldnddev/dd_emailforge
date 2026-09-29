//! Apply an insert-picker kind at the current tree selection.
use crate::model::{BodyNode, ColumnChild, MjColumn, MjGroup, MjSection, SectionChild, Template};

use super::App;
use super::component_kind::{
    ComponentKind, SelectionClass, classify, empty_column, empty_group, wrap_leaf_in_section,
};
use super::edits::rebalance_widths;
use super::toasts::ToastLevel;
use super::tree::{Step, TreeId};

impl App {
    pub(in crate::tui) fn open_insert_picker(&mut self) {
        if self.template.is_none() {
            self.push_toast(ToastLevel::Warning, super::EMPTY_TEMPLATE_HINT);
            return;
        }
        let class = classify(
            self.template.as_ref(),
            &self.selected_tree_id().unwrap_or(TreeId::Body),
        );
        let rows = super::component_kind::picker_rows(class, "");
        let selected = super::component_kind::first_kind_index(&rows);
        self.modal = Some(super::Modal::ComponentPicker {
            query: String::new(),
            selected,
        });
    }

    pub(in crate::tui) fn insert_kind(&mut self, kind: ComponentKind) {
        let Some(id) = self.selected_tree_id() else {
            self.push_toast(ToastLevel::Warning, "Nothing selected.");
            return;
        };
        let class = classify(self.template.as_ref(), &id);
        if !kind.legal_for(class) {
            self.push_toast(
                ToastLevel::Warning,
                format!("Cannot insert {} here", kind.label()),
            );
            return;
        }
        self.push_undo();
        let Some(template) = self.template.as_mut() else {
            self.undo_stack.pop();
            return;
        };
        match insert_into(template, &id, kind) {
            Ok((new_id, wrapped)) => {
                self.collapsed.remove(&TreeId::Body);
                expand_ancestors(&mut self.collapsed, &new_id);
                self.select_tree_id(&new_id);
                if wrapped {
                    self.push_toast(ToastLevel::Info, "Wrapped in mj-section");
                } else {
                    self.push_toast(ToastLevel::Success, format!("Inserted {}.", kind.label()));
                }
            }
            Err(_) => {
                self.undo_stack.pop();
                self.push_toast(
                    ToastLevel::Warning,
                    format!("Cannot insert {} here", kind.label()),
                );
            }
        }
    }
}

pub(in crate::tui) fn expand_ancestors(
    collapsed: &mut std::collections::HashSet<TreeId>,
    id: &TreeId,
) {
    collapsed.remove(&TreeId::Body);
    let TreeId::Path(path) = id else {
        return;
    };
    for i in 1..path.len() {
        collapsed.remove(&TreeId::Path(path[..i].to_vec()));
    }
}

fn insert_into(
    t: &mut Template,
    id: &TreeId,
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    match id {
        TreeId::Head | TreeId::Brand => Err("illegal"),
        TreeId::Body => insert_body(t, t.body.nodes.len(), kind),
        TreeId::Path(path) => insert_at_path(t, path, kind),
    }
}

fn insert_body(
    t: &mut Template,
    at: usize,
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    if let Some(node) = kind.as_body_node() {
        let at = at.min(t.body.nodes.len());
        t.body.nodes.insert(at, node);
        Ok((TreeId::Path(vec![Step::BodyNode(at)]), false))
    } else if let Some(leaf) = kind.as_leaf() {
        let at = at.min(t.body.nodes.len());
        t.body.nodes.insert(at, wrap_leaf_in_section(leaf));
        Ok((
            TreeId::Path(vec![
                Step::BodyNode(at),
                Step::SectionChild(0),
                Step::ColComp(0),
            ]),
            true,
        ))
    } else {
        Err("illegal")
    }
}

fn insert_at_path(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let class = classify(Some(t), &TreeId::Path(path.to_vec()));
    match class {
        SelectionClass::EmailBlock => {
            let Step::BodyNode(i) = path.first().copied().ok_or("illegal")? else {
                return Err("illegal");
            };
            insert_body(t, i + 1, kind)
        }
        SelectionClass::Section { .. } => insert_on_section(t, path, kind),
        SelectionClass::Wrapper => insert_on_wrapper(t, path, kind),
        SelectionClass::Hero => insert_on_hero(t, path, kind),
        SelectionClass::Group => insert_on_group(t, path, kind),
        SelectionClass::Column => insert_on_column(t, path, kind),
        SelectionClass::Leaf => insert_after_leaf(t, path, kind),
        SelectionClass::Navbar => insert_on_navbar(t, path, kind),
        SelectionClass::Accordion => insert_on_accordion(t, path, kind),
        SelectionClass::Carousel => insert_on_carousel(t, path, kind),
        SelectionClass::NavbarLink => insert_after_navbar_link(t, path, kind),
        SelectionClass::AccordionEl => insert_after_accordion_el(t, path, kind),
        SelectionClass::CarouselImg => insert_after_carousel_img(t, path, kind),
        SelectionClass::Body => insert_body(t, t.body.nodes.len(), kind),
        SelectionClass::HeadBrand => Err("illegal"),
    }
}

fn insert_on_section(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    if kind.is_block() || kind.is_layout() {
        return insert_after_body_node(t, path, kind);
    }
    if kind == ComponentKind::MjGroup {
        let s = section_mut(t, path).ok_or("illegal")?;
        s.children.push(SectionChild::MjGroup(empty_group()));
        let idx = s.children.len() - 1;
        let mut p = path.to_vec();
        p.push(Step::SectionChild(idx));
        return Ok((TreeId::Path(p), false));
    }
    if kind == ComponentKind::MjColumn {
        let s = section_mut(t, path).ok_or("illegal")?;
        s.children.push(SectionChild::MjColumn(empty_column()));
        rebalance_section(s);
        let idx = s.children.len() - 1;
        let mut p = path.to_vec();
        p.push(Step::SectionChild(idx));
        return Ok((TreeId::Path(p), false));
    }
    if let Some(leaf) = kind.as_leaf() {
        return insert_leaf_in_section(t, path, leaf);
    }
    Err("illegal")
}

fn insert_after_body_node(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    match path {
        [Step::BodyNode(i)] => insert_body(t, *i + 1, kind),
        [prefix @ .., Step::WrapperChild(j)] => {
            let parent = body_node_mut(t, prefix).ok_or("illegal")?;
            let BodyNode::MjWrapper(w) = parent else {
                return Err("illegal");
            };
            let node = kind.as_body_node().ok_or("illegal")?;
            if !matches!(node, BodyNode::MjSection(_) | BodyNode::MjHero(_)) {
                return Err("illegal");
            }
            let at = (*j + 1).min(w.children.len());
            w.children.insert(at, node);
            let mut p = prefix.to_vec();
            p.push(Step::WrapperChild(at));
            Ok((TreeId::Path(p), false))
        }
        _ => Err("illegal"),
    }
}

fn insert_on_wrapper(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let node = kind.as_body_node().ok_or("illegal")?;
    if !matches!(node, BodyNode::MjSection(_) | BodyNode::MjHero(_)) {
        return Err("illegal");
    }
    let parent = body_node_mut(t, path).ok_or("illegal")?;
    let BodyNode::MjWrapper(w) = parent else {
        return Err("illegal");
    };
    w.children.push(node);
    let idx = w.children.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::WrapperChild(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_on_hero(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let leaf = kind.as_leaf().ok_or("illegal")?;
    let parent = body_node_mut(t, path).ok_or("illegal")?;
    let BodyNode::MjHero(h) = parent else {
        return Err("illegal");
    };
    h.children.push(leaf);
    let idx = h.children.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::HeroChild(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_on_group(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    if kind != ComponentKind::MjColumn {
        return Err("illegal");
    }
    let g = group_mut(t, path).ok_or("illegal")?;
    g.children.push(empty_column());
    rebalance_group(g);
    let idx = g.children.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::GroupCol(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_on_column(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    if kind == ComponentKind::MjColumn {
        return insert_column_after(t, path);
    }
    let leaf = kind.as_leaf().ok_or("illegal")?;
    let col = column_mut(t, path).ok_or("illegal")?;
    col.components.push(leaf);
    let idx = col.components.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::ColComp(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_after_leaf(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let leaf = kind.as_leaf().ok_or("illegal")?;
    let (last, parent) = path.split_last().ok_or("illegal")?;
    match last {
        Step::ColComp(i) => {
            let col = column_mut(t, parent).ok_or("illegal")?;
            let at = (*i + 1).min(col.components.len());
            col.components.insert(at, leaf);
            let mut p = parent.to_vec();
            p.push(Step::ColComp(at));
            Ok((TreeId::Path(p), false))
        }
        Step::HeroChild(i) => {
            let parent_node = body_node_mut(t, parent).ok_or("illegal")?;
            let BodyNode::MjHero(h) = parent_node else {
                return Err("illegal");
            };
            let at = (*i + 1).min(h.children.len());
            h.children.insert(at, leaf);
            let mut p = parent.to_vec();
            p.push(Step::HeroChild(at));
            Ok((TreeId::Path(p), false))
        }
        _ => Err("illegal"),
    }
}

fn insert_leaf_in_section(
    t: &mut Template,
    path: &[Step],
    leaf: ColumnChild,
) -> Result<(TreeId, bool), &'static str> {
    let s = section_mut(t, path).ok_or("illegal")?;
    if s.children.is_empty() {
        let mut col = empty_column();
        col.width = Some("100%".into());
        col.components.push(leaf);
        s.children.push(SectionChild::MjColumn(col));
        let mut p = path.to_vec();
        p.push(Step::SectionChild(0));
        p.push(Step::ColComp(0));
        return Ok((TreeId::Path(p), false));
    }
    let last_i = s.children.len() - 1;
    match &mut s.children[last_i] {
        SectionChild::MjColumn(c) => {
            c.components.push(leaf);
            let comp_i = c.components.len() - 1;
            let mut p = path.to_vec();
            p.push(Step::SectionChild(last_i));
            p.push(Step::ColComp(comp_i));
            Ok((TreeId::Path(p), false))
        }
        SectionChild::MjGroup(g) => {
            if g.children.is_empty() {
                g.children.push(empty_column());
            }
            let col_i = g.children.len() - 1;
            g.children[col_i].components.push(leaf);
            let comp_i = g.children[col_i].components.len() - 1;
            let mut p = path.to_vec();
            p.push(Step::SectionChild(last_i));
            p.push(Step::GroupCol(col_i));
            p.push(Step::ColComp(comp_i));
            Ok((TreeId::Path(p), false))
        }
    }
}

fn insert_on_navbar(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let link = kind.as_navbar_link().ok_or("illegal")?;
    let ColumnChild::MjNavbar(nav) = column_child_mut(t, path).ok_or("illegal")? else {
        return Err("illegal");
    };
    nav.links.push(link);
    let idx = nav.links.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::NavbarLink(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_on_accordion(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let el = kind.as_accordion_element().ok_or("illegal")?;
    let ColumnChild::MjAccordion(acc) = column_child_mut(t, path).ok_or("illegal")? else {
        return Err("illegal");
    };
    acc.elements.push(el);
    let idx = acc.elements.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::AccordionEl(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_on_carousel(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let img = kind.as_carousel_image().ok_or("illegal")?;
    let ColumnChild::MjCarousel(car) = column_child_mut(t, path).ok_or("illegal")? else {
        return Err("illegal");
    };
    car.images.push(img);
    let idx = car.images.len() - 1;
    let mut p = path.to_vec();
    p.push(Step::CarouselImg(idx));
    Ok((TreeId::Path(p), false))
}

fn insert_after_navbar_link(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let link = kind.as_navbar_link().ok_or("illegal")?;
    let (last, parent) = path.split_last().ok_or("illegal")?;
    let Step::NavbarLink(i) = last else {
        return Err("illegal");
    };
    let ColumnChild::MjNavbar(nav) = column_child_mut(t, parent).ok_or("illegal")? else {
        return Err("illegal");
    };
    let at = (*i + 1).min(nav.links.len());
    nav.links.insert(at, link);
    let mut p = parent.to_vec();
    p.push(Step::NavbarLink(at));
    Ok((TreeId::Path(p), false))
}

fn insert_after_accordion_el(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let el = kind.as_accordion_element().ok_or("illegal")?;
    let (last, parent) = path.split_last().ok_or("illegal")?;
    let Step::AccordionEl(i) = last else {
        return Err("illegal");
    };
    let ColumnChild::MjAccordion(acc) = column_child_mut(t, parent).ok_or("illegal")? else {
        return Err("illegal");
    };
    let at = (*i + 1).min(acc.elements.len());
    acc.elements.insert(at, el);
    let mut p = parent.to_vec();
    p.push(Step::AccordionEl(at));
    Ok((TreeId::Path(p), false))
}

fn insert_after_carousel_img(
    t: &mut Template,
    path: &[Step],
    kind: ComponentKind,
) -> Result<(TreeId, bool), &'static str> {
    let img = kind.as_carousel_image().ok_or("illegal")?;
    let (last, parent) = path.split_last().ok_or("illegal")?;
    let Step::CarouselImg(i) = last else {
        return Err("illegal");
    };
    let ColumnChild::MjCarousel(car) = column_child_mut(t, parent).ok_or("illegal")? else {
        return Err("illegal");
    };
    let at = (*i + 1).min(car.images.len());
    car.images.insert(at, img);
    let mut p = parent.to_vec();
    p.push(Step::CarouselImg(at));
    Ok((TreeId::Path(p), false))
}

fn column_child_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut ColumnChild> {
    let (last, parent) = path.split_last()?;
    match last {
        Step::ColComp(i) => column_mut(t, parent)?.components.get_mut(*i),
        Step::HeroChild(i) => {
            let parent_node = body_node_mut(t, parent)?;
            let BodyNode::MjHero(h) = parent_node else {
                return None;
            };
            h.children.get_mut(*i)
        }
        _ => None,
    }
}

fn insert_column_after(t: &mut Template, path: &[Step]) -> Result<(TreeId, bool), &'static str> {
    let (last, parent) = path.split_last().ok_or("illegal")?;
    match last {
        Step::SectionChild(i) => {
            let s = section_mut(t, parent).ok_or("illegal")?;
            let at = (*i + 1).min(s.children.len());
            s.children
                .insert(at, SectionChild::MjColumn(empty_column()));
            rebalance_section(s);
            let mut p = parent.to_vec();
            p.push(Step::SectionChild(at));
            Ok((TreeId::Path(p), false))
        }
        Step::GroupCol(i) => {
            let g = group_mut(t, parent).ok_or("illegal")?;
            let at = (*i + 1).min(g.children.len());
            g.children.insert(at, empty_column());
            rebalance_group(g);
            let mut p = parent.to_vec();
            p.push(Step::GroupCol(at));
            Ok((TreeId::Path(p), false))
        }
        _ => Err("illegal"),
    }
}

fn rebalance_section(s: &mut MjSection) {
    let n = s
        .children
        .iter()
        .filter(|c| matches!(c, SectionChild::MjColumn(_)))
        .count();
    let widths = rebalance_widths(n);
    let mut i = 0;
    for child in &mut s.children {
        if let SectionChild::MjColumn(c) = child {
            c.width = Some(widths[i].clone());
            i += 1;
        }
    }
}

fn rebalance_group(g: &mut MjGroup) {
    let widths = rebalance_widths(g.children.len());
    for (c, w) in g.children.iter_mut().zip(widths) {
        c.width = Some(w);
    }
}

fn body_node_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut BodyNode> {
    let (first, rest) = path.split_first()?;
    let Step::BodyNode(idx) = first else {
        return None;
    };
    walk_node_mut(t.body.nodes.get_mut(*idx)?, rest)
}

fn walk_node_mut<'a>(node: &'a mut BodyNode, path: &[Step]) -> Option<&'a mut BodyNode> {
    if path.is_empty() {
        return Some(node);
    }
    let (first, rest) = path.split_first()?;
    match (node, first) {
        (BodyNode::MjWrapper(w), Step::WrapperChild(i)) => {
            walk_node_mut(w.children.get_mut(*i)?, rest)
        }
        _ => None,
    }
}

fn section_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut MjSection> {
    match body_node_mut(t, path)? {
        BodyNode::MjSection(s) => Some(s),
        _ => None,
    }
}

fn group_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut MjGroup> {
    let (last, parent) = path.split_last()?;
    let Step::SectionChild(i) = last else {
        return None;
    };
    match section_mut(t, parent)?.children.get_mut(*i)? {
        SectionChild::MjGroup(g) => Some(g),
        _ => None,
    }
}

fn column_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut MjColumn> {
    let (last, parent) = path.split_last()?;
    match last {
        Step::SectionChild(i) => match section_mut(t, parent)?.children.get_mut(*i)? {
            SectionChild::MjColumn(c) => Some(c),
            _ => None,
        },
        Step::GroupCol(i) => group_mut(t, parent)?.children.get_mut(*i),
        _ => None,
    }
}
