//! Undo, delete, duplicate, reorder, and column add/remove.
use crate::model::{BodyNode, MjColumn, MjGroup, MjSection, SectionChild, Template};

use super::App;
use super::toasts::ToastLevel;
use super::tree::{Step, TreeId};

const UNDO_CAP: usize = 20;

impl App {
    pub(in crate::tui) fn push_undo(&mut self) {
        let Some(template) = self.template.clone() else {
            return;
        };
        self.undo_stack.push(template);
        if self.undo_stack.len() > UNDO_CAP {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub(in crate::tui) fn undo_last(&mut self) {
        let Some(prev) = self.undo_stack.pop() else {
            self.push_toast(ToastLevel::Warning, "Nothing to undo.");
            return;
        };
        if let Some(current) = self.template.take() {
            self.redo_stack.push(current);
            if self.redo_stack.len() > UNDO_CAP {
                self.redo_stack.remove(0);
            }
        }
        self.template = Some(prev);
        self.clamp_tree_selection();
        self.write_mjml_sidecar();
        self.push_toast(ToastLevel::Success, "Undid last change.");
    }

    pub(in crate::tui) fn redo_last(&mut self) {
        let Some(next) = self.redo_stack.pop() else {
            self.push_toast(ToastLevel::Warning, "Nothing to redo.");
            return;
        };
        if let Some(current) = self.template.take() {
            self.undo_stack.push(current);
            if self.undo_stack.len() > UNDO_CAP {
                self.undo_stack.remove(0);
            }
        }
        self.template = Some(next);
        self.clamp_tree_selection();
        self.write_mjml_sidecar();
        self.push_toast(ToastLevel::Success, "Redid last change.");
    }

    pub(in crate::tui) fn select_tree_id(&mut self, id: &TreeId) {
        let rows = self.tree_rows();
        if let Some(i) = rows.iter().position(|r| r.id == *id) {
            self.selected_row = i;
        } else {
            self.clamp_tree_selection();
        }
    }

    pub(in crate::tui) fn selected_tree_id(&self) -> Option<TreeId> {
        self.tree_rows()
            .get(self.selected_row)
            .map(|r| r.id.clone())
    }

    pub(in crate::tui) fn delete_selected_row(&mut self) {
        let Some(id) = self.selected_tree_id() else {
            self.push_toast(ToastLevel::Warning, "Nothing selected to delete.");
            return;
        };
        match &id {
            TreeId::Head | TreeId::Brand | TreeId::Body => {
                self.push_toast(ToastLevel::Warning, "Cannot delete this row.");
                return;
            }
            TreeId::Path(path) => {
                if is_last_column(self.template.as_ref(), path) {
                    self.push_toast(ToastLevel::Info, "A section needs at least one column");
                    return;
                }
                let path = path.clone();
                self.push_undo();
                let Some(template) = self.template.as_mut() else {
                    self.undo_stack.pop();
                    return;
                };
                if !remove_path(template, &path) {
                    self.undo_stack.pop();
                    self.push_toast(ToastLevel::Warning, "Could not delete that row.");
                    return;
                }
                self.clamp_tree_selection();
                self.push_toast(ToastLevel::Info, "Deleted.");
            }
        }
    }

    pub(in crate::tui) fn duplicate_selected_row(&mut self) {
        let Some(id) = self.selected_tree_id() else {
            self.push_toast(ToastLevel::Warning, "Nothing selected to duplicate.");
            return;
        };
        let TreeId::Path(path) = id else {
            self.push_toast(ToastLevel::Warning, "Cannot duplicate this row.");
            return;
        };
        self.push_undo();
        let Some(template) = self.template.as_mut() else {
            self.undo_stack.pop();
            return;
        };
        let Some(new_path) = duplicate_path(template, &path) else {
            self.undo_stack.pop();
            self.push_toast(ToastLevel::Warning, "Could not duplicate that row.");
            return;
        };
        self.select_tree_id(&TreeId::Path(new_path));
        self.push_toast(ToastLevel::Success, "Duplicated.");
    }

    pub(in crate::tui) fn reorder_selected(&mut self, delta: isize) {
        let Some(id) = self.selected_tree_id() else {
            return;
        };
        let TreeId::Path(path) = id else {
            self.push_toast(ToastLevel::Info, "Cannot reorder this row.");
            return;
        };
        self.push_undo();
        let Some(template) = self.template.as_mut() else {
            self.undo_stack.pop();
            return;
        };
        let Some(new_path) = reorder_path(template, &path, delta) else {
            self.undo_stack.pop();
            return;
        };
        self.select_tree_id(&TreeId::Path(new_path));
        self.push_toast(ToastLevel::Success, "Reordered.");
    }

    pub(in crate::tui) fn add_column(&mut self) {
        let Some(id) = self.selected_tree_id() else {
            return;
        };
        let Some(owner) = enclosing_columns(self.template.as_ref(), &id) else {
            self.push_toast(ToastLevel::Info, "No columns on this selection");
            return;
        };
        self.push_undo();
        let Some(template) = self.template.as_mut() else {
            self.undo_stack.pop();
            return;
        };
        let Some(new_id) = insert_column(template, &owner) else {
            self.undo_stack.pop();
            self.push_toast(ToastLevel::Warning, "Could not add a column.");
            return;
        };
        self.collapsed
            .remove(&TreeId::Path(owner.owner_path.clone()));
        if owner.owner_path.len() == 1 {
            self.collapsed.remove(&TreeId::Body);
        }
        self.select_tree_id(&new_id);
        self.push_toast(ToastLevel::Success, "Added column.");
    }

    pub(in crate::tui) fn remove_column(&mut self) {
        let Some(id) = self.selected_tree_id() else {
            return;
        };
        let Some(owner) = enclosing_columns(self.template.as_ref(), &id) else {
            self.push_toast(ToastLevel::Info, "No columns on this selection");
            return;
        };
        let n = column_count(self.template.as_ref(), &owner).unwrap_or(0);
        if n <= 1 {
            self.push_toast(ToastLevel::Info, "A section needs at least one column");
            return;
        }
        self.push_undo();
        let Some(template) = self.template.as_mut() else {
            self.undo_stack.pop();
            return;
        };
        if !delete_current_column(template, &owner) {
            self.undo_stack.pop();
            self.push_toast(ToastLevel::Warning, "Could not remove that column.");
            return;
        }
        self.clamp_tree_selection();
        self.push_toast(ToastLevel::Info, "Removed column.");
    }

    pub(in crate::tui) fn hop_column(&mut self, delta: isize) {
        let Some(id) = self.selected_tree_id() else {
            return;
        };
        let Some(owner) = enclosing_columns(self.template.as_ref(), &id) else {
            self.push_toast(ToastLevel::Info, "No columns on this selection");
            return;
        };
        let n = column_count(self.template.as_ref(), &owner).unwrap_or(0) as isize;
        if n == 0 {
            return;
        }
        let next = (owner.col_index as isize + delta).rem_euclid(n) as usize;
        let mut path = owner.owner_path;
        match owner.kind {
            ColOwnerKind::Section => path.push(Step::SectionChild(next)),
            ColOwnerKind::Group => path.push(Step::GroupCol(next)),
        }
        self.collapsed
            .remove(&TreeId::Path(path[..path.len().saturating_sub(1)].to_vec()));
        self.select_tree_id(&TreeId::Path(path));
    }
}

#[derive(Clone, Debug)]
struct ColOwner {
    owner_path: Vec<Step>,
    col_index: usize,
    kind: ColOwnerKind,
}

#[derive(Clone, Copy, Debug)]
enum ColOwnerKind {
    Section,
    Group,
}

fn empty_column() -> MjColumn {
    MjColumn::default()
}

pub(super) fn rebalance_widths(n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let each = 100 / n;
    let last = 100 - each * (n - 1);
    (0..n)
        .map(|i| {
            let p = if i + 1 == n { last } else { each };
            format!("{p}%")
        })
        .collect()
}

fn apply_rebalance_section(s: &mut MjSection) {
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

fn apply_rebalance_group(g: &mut MjGroup) {
    let widths = rebalance_widths(g.children.len());
    for (c, w) in g.children.iter_mut().zip(widths) {
        c.width = Some(w);
    }
}

fn enclosing_columns(template: Option<&Template>, id: &TreeId) -> Option<ColOwner> {
    let TreeId::Path(path) = id else {
        return None;
    };
    let t = template?;
    let mut owner: Option<ColOwner> = None;
    let mut node_path: Vec<Step> = Vec::new();
    let mut node = None::<&BodyNode>;
    for (i, step) in path.iter().enumerate() {
        match step {
            Step::BodyNode(idx) if i == 0 => {
                node_path.push(*step);
                node = t.body.nodes.get(*idx);
                if let Some(BodyNode::MjSection(_)) = node {
                    owner = Some(ColOwner {
                        owner_path: node_path.clone(),
                        col_index: 0,
                        kind: ColOwnerKind::Section,
                    });
                }
            }
            Step::WrapperChild(idx) => {
                let BodyNode::MjWrapper(w) = node? else {
                    return owner;
                };
                node_path.push(*step);
                node = w.children.get(*idx);
                if let Some(BodyNode::MjSection(_)) = node {
                    owner = Some(ColOwner {
                        owner_path: node_path.clone(),
                        col_index: 0,
                        kind: ColOwnerKind::Section,
                    });
                }
            }
            Step::SectionChild(idx) => {
                let BodyNode::MjSection(s) = node? else {
                    return owner;
                };
                match s.children.get(*idx)? {
                    SectionChild::MjColumn(_) => {
                        owner = Some(ColOwner {
                            owner_path: node_path.clone(),
                            col_index: *idx,
                            kind: ColOwnerKind::Section,
                        });
                    }
                    SectionChild::MjGroup(_) => {
                        let mut gp = node_path.clone();
                        gp.push(*step);
                        owner = Some(ColOwner {
                            owner_path: gp,
                            col_index: 0,
                            kind: ColOwnerKind::Group,
                        });
                    }
                }
            }
            Step::GroupCol(idx) => {
                if let Some(o) = owner.as_mut() {
                    if matches!(o.kind, ColOwnerKind::Group) {
                        o.col_index = *idx;
                    }
                }
            }
            Step::ColComp(_)
            | Step::HeroChild(_)
            | Step::BodyNode(_)
            | Step::NavbarLink(_)
            | Step::AccordionEl(_)
            | Step::CarouselImg(_) => {}
        }
    }
    owner
}

fn node_at<'a>(t: &'a Template, path: &[Step]) -> Option<&'a BodyNode> {
    let mut node = None;
    for (i, step) in path.iter().enumerate() {
        match step {
            Step::BodyNode(idx) if i == 0 => node = t.body.nodes.get(*idx),
            Step::WrapperChild(idx) => {
                let BodyNode::MjWrapper(w) = node? else {
                    return None;
                };
                node = w.children.get(*idx);
            }
            _ => return None,
        }
    }
    node
}

fn node_at_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut BodyNode> {
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

fn column_count(template: Option<&Template>, owner: &ColOwner) -> Option<usize> {
    let t = template?;
    match owner.kind {
        ColOwnerKind::Section => {
            let BodyNode::MjSection(s) = node_at(t, &owner.owner_path)? else {
                return None;
            };
            Some(s.children.len())
        }
        ColOwnerKind::Group => {
            let g = group_at(t, &owner.owner_path)?;
            Some(g.children.len())
        }
    }
}

fn group_at<'a>(t: &'a Template, path: &[Step]) -> Option<&'a MjGroup> {
    let (last, parent) = path.split_last()?;
    let Step::SectionChild(i) = last else {
        return None;
    };
    let BodyNode::MjSection(s) = node_at(t, parent)? else {
        return None;
    };
    match s.children.get(*i)? {
        SectionChild::MjGroup(g) => Some(g),
        _ => None,
    }
}

fn insert_column(t: &mut Template, owner: &ColOwner) -> Option<TreeId> {
    match owner.kind {
        ColOwnerKind::Section => {
            let BodyNode::MjSection(s) = node_at_mut(t, &owner.owner_path)? else {
                return None;
            };
            s.children.push(SectionChild::MjColumn(empty_column()));
            apply_rebalance_section(s);
            let idx = s.children.len() - 1;
            let mut path = owner.owner_path.clone();
            path.push(Step::SectionChild(idx));
            Some(TreeId::Path(path))
        }
        ColOwnerKind::Group => {
            let g = group_at_mut(t, &owner.owner_path)?;
            g.children.push(empty_column());
            apply_rebalance_group(g);
            let idx = g.children.len() - 1;
            let mut path = owner.owner_path.clone();
            path.push(Step::GroupCol(idx));
            Some(TreeId::Path(path))
        }
    }
}

fn group_at_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<&'a mut MjGroup> {
    let (last, parent) = path.split_last()?;
    let Step::SectionChild(i) = last else {
        return None;
    };
    let BodyNode::MjSection(s) = node_at_mut(t, parent)? else {
        return None;
    };
    match s.children.get_mut(*i)? {
        SectionChild::MjGroup(g) => Some(g),
        _ => None,
    }
}

fn delete_current_column(t: &mut Template, owner: &ColOwner) -> bool {
    match owner.kind {
        ColOwnerKind::Section => {
            let Some(BodyNode::MjSection(s)) = node_at_mut(t, &owner.owner_path) else {
                return false;
            };
            if owner.col_index >= s.children.len() || s.children.len() <= 1 {
                return false;
            }
            s.children.remove(owner.col_index);
            apply_rebalance_section(s);
            true
        }
        ColOwnerKind::Group => {
            let Some(g) = group_at_mut(t, &owner.owner_path) else {
                return false;
            };
            if owner.col_index >= g.children.len() || g.children.len() <= 1 {
                return false;
            }
            g.children.remove(owner.col_index);
            apply_rebalance_group(g);
            true
        }
    }
}

fn is_last_column(template: Option<&Template>, path: &[Step]) -> bool {
    let id = TreeId::Path(path.to_vec());
    let Some(owner) = enclosing_columns(template, &id) else {
        return false;
    };
    let ends_on_column = matches!(path.last(), Some(Step::SectionChild(_) | Step::GroupCol(_)));
    if !ends_on_column {
        return false;
    }
    column_count(template, &owner) == Some(1)
}

fn remove_path(t: &mut Template, path: &[Step]) -> bool {
    match path {
        [Step::BodyNode(i)] => {
            if *i < t.body.nodes.len() {
                t.body.nodes.remove(*i);
                true
            } else {
                false
            }
        }
        [Step::BodyNode(i), rest @ ..] => t
            .body
            .nodes
            .get_mut(*i)
            .map(|n| remove_from_body_node(n, rest))
            .unwrap_or(false),
        _ => false,
    }
}

fn remove_from_body_node(n: &mut BodyNode, rest: &[Step]) -> bool {
    match (n, rest) {
        (BodyNode::MjWrapper(w), [Step::WrapperChild(j)]) => {
            if *j < w.children.len() {
                w.children.remove(*j);
                true
            } else {
                false
            }
        }
        (BodyNode::MjWrapper(w), [Step::WrapperChild(j), tail @ ..]) => w
            .children
            .get_mut(*j)
            .map(|c| remove_from_body_node(c, tail))
            .unwrap_or(false),
        (BodyNode::MjSection(s), [Step::SectionChild(j)]) => {
            if *j < s.children.len() {
                s.children.remove(*j);
                true
            } else {
                false
            }
        }
        (BodyNode::MjSection(s), [Step::SectionChild(j), tail @ ..]) => {
            match s.children.get_mut(*j) {
                Some(SectionChild::MjColumn(c)) => remove_from_column(c, tail),
                Some(SectionChild::MjGroup(g)) => remove_from_group(g, tail),
                None => false,
            }
        }
        (BodyNode::MjHero(h), [Step::HeroChild(j)]) => {
            if *j < h.children.len() {
                h.children.remove(*j);
                true
            } else {
                false
            }
        }
        (BodyNode::MjHero(h), [Step::HeroChild(j), tail @ ..]) => h
            .children
            .get_mut(*j)
            .map(|c| remove_from_column_child(c, tail))
            .unwrap_or(false),
        _ => false,
    }
}

fn remove_from_column(c: &mut MjColumn, rest: &[Step]) -> bool {
    match rest {
        [Step::ColComp(i)] if *i < c.components.len() => {
            c.components.remove(*i);
            true
        }
        [Step::ColComp(i), tail @ ..] => c
            .components
            .get_mut(*i)
            .map(|ch| remove_from_column_child(ch, tail))
            .unwrap_or(false),
        _ => false,
    }
}

fn remove_from_column_child(c: &mut crate::model::ColumnChild, rest: &[Step]) -> bool {
    match (c, rest) {
        (crate::model::ColumnChild::MjNavbar(n), [Step::NavbarLink(i)]) if *i < n.links.len() => {
            n.links.remove(*i);
            true
        }
        (crate::model::ColumnChild::MjAccordion(a), [Step::AccordionEl(i)])
            if *i < a.elements.len() =>
        {
            a.elements.remove(*i);
            true
        }
        (crate::model::ColumnChild::MjCarousel(car), [Step::CarouselImg(i)])
            if *i < car.images.len() =>
        {
            car.images.remove(*i);
            true
        }
        _ => false,
    }
}

fn remove_from_group(g: &mut MjGroup, rest: &[Step]) -> bool {
    match rest {
        [Step::GroupCol(i)] if *i < g.children.len() => {
            g.children.remove(*i);
            true
        }
        [Step::GroupCol(i), tail @ ..] => g
            .children
            .get_mut(*i)
            .map(|c| remove_from_column(c, tail))
            .unwrap_or(false),
        _ => false,
    }
}

fn duplicate_path(t: &mut Template, path: &[Step]) -> Option<Vec<Step>> {
    match path {
        [Step::BodyNode(i)] => {
            if *i >= t.body.nodes.len() {
                return None;
            }
            let clone = t.body.nodes[*i].clone();
            t.body.nodes.insert(*i + 1, clone);
            Some(vec![Step::BodyNode(*i + 1)])
        }
        [Step::BodyNode(i), rest @ ..] => {
            let n = t.body.nodes.get_mut(*i)?;
            let mut new_rest = duplicate_in_body_node(n, rest)?;
            let mut out = vec![Step::BodyNode(*i)];
            out.append(&mut new_rest);
            Some(out)
        }
        _ => None,
    }
}

fn duplicate_in_body_node(n: &mut BodyNode, rest: &[Step]) -> Option<Vec<Step>> {
    match (n, rest) {
        (BodyNode::MjWrapper(w), [Step::WrapperChild(j)]) => {
            if *j >= w.children.len() {
                return None;
            }
            let clone = w.children[*j].clone();
            w.children.insert(*j + 1, clone);
            Some(vec![Step::WrapperChild(*j + 1)])
        }
        (BodyNode::MjWrapper(w), [Step::WrapperChild(j), tail @ ..]) => {
            let child = w.children.get_mut(*j)?;
            let mut rest = duplicate_in_body_node(child, tail)?;
            let mut out = vec![Step::WrapperChild(*j)];
            out.append(&mut rest);
            Some(out)
        }
        (BodyNode::MjSection(s), [Step::SectionChild(j)]) => {
            if *j >= s.children.len() {
                return None;
            }
            let clone = s.children[*j].clone();
            s.children.insert(*j + 1, clone);
            Some(vec![Step::SectionChild(*j + 1)])
        }
        (BodyNode::MjSection(s), [Step::SectionChild(j), tail @ ..]) => {
            match s.children.get_mut(*j)? {
                SectionChild::MjColumn(c) => {
                    let mut rest = duplicate_in_column(c, tail)?;
                    let mut out = vec![Step::SectionChild(*j)];
                    out.append(&mut rest);
                    Some(out)
                }
                SectionChild::MjGroup(g) => {
                    let mut rest = duplicate_in_group(g, tail)?;
                    let mut out = vec![Step::SectionChild(*j)];
                    out.append(&mut rest);
                    Some(out)
                }
            }
        }
        (BodyNode::MjHero(h), [Step::HeroChild(j)]) => {
            if *j >= h.children.len() {
                return None;
            }
            let clone = h.children[*j].clone();
            h.children.insert(*j + 1, clone);
            Some(vec![Step::HeroChild(*j + 1)])
        }
        (BodyNode::MjHero(h), [Step::HeroChild(j), tail @ ..]) => {
            let child = h.children.get_mut(*j)?;
            let mut rest = duplicate_in_column_child(child, tail)?;
            let mut out = vec![Step::HeroChild(*j)];
            out.append(&mut rest);
            Some(out)
        }
        _ => None,
    }
}

fn duplicate_in_column(c: &mut MjColumn, rest: &[Step]) -> Option<Vec<Step>> {
    match rest {
        [Step::ColComp(i)] if *i < c.components.len() => {
            let clone = c.components[*i].clone();
            c.components.insert(*i + 1, clone);
            Some(vec![Step::ColComp(*i + 1)])
        }
        [Step::ColComp(i), tail @ ..] => {
            let child = c.components.get_mut(*i)?;
            let mut rest = duplicate_in_column_child(child, tail)?;
            let mut out = vec![Step::ColComp(*i)];
            out.append(&mut rest);
            Some(out)
        }
        _ => None,
    }
}

fn duplicate_in_column_child(
    c: &mut crate::model::ColumnChild,
    rest: &[Step],
) -> Option<Vec<Step>> {
    match (c, rest) {
        (crate::model::ColumnChild::MjNavbar(n), [Step::NavbarLink(i)]) if *i < n.links.len() => {
            let clone = n.links[*i].clone();
            n.links.insert(*i + 1, clone);
            Some(vec![Step::NavbarLink(*i + 1)])
        }
        (crate::model::ColumnChild::MjAccordion(a), [Step::AccordionEl(i)])
            if *i < a.elements.len() =>
        {
            let clone = a.elements[*i].clone();
            a.elements.insert(*i + 1, clone);
            Some(vec![Step::AccordionEl(*i + 1)])
        }
        (crate::model::ColumnChild::MjCarousel(car), [Step::CarouselImg(i)])
            if *i < car.images.len() =>
        {
            let clone = car.images[*i].clone();
            car.images.insert(*i + 1, clone);
            Some(vec![Step::CarouselImg(*i + 1)])
        }
        _ => None,
    }
}

fn duplicate_in_group(g: &mut MjGroup, rest: &[Step]) -> Option<Vec<Step>> {
    match rest {
        [Step::GroupCol(i)] if *i < g.children.len() => {
            let clone = g.children[*i].clone();
            g.children.insert(*i + 1, clone);
            Some(vec![Step::GroupCol(*i + 1)])
        }
        [Step::GroupCol(i), tail @ ..] => {
            let c = g.children.get_mut(*i)?;
            let mut rest = duplicate_in_column(c, tail)?;
            let mut out = vec![Step::GroupCol(*i)];
            out.append(&mut rest);
            Some(out)
        }
        _ => None,
    }
}

fn reorder_path(t: &mut Template, path: &[Step], delta: isize) -> Option<Vec<Step>> {
    match path {
        [Step::BodyNode(i)] => {
            swap_index(&mut t.body.nodes, *i, delta).map(|ni| vec![Step::BodyNode(ni)])
        }
        [Step::BodyNode(i), rest @ ..] => {
            let n = t.body.nodes.get_mut(*i)?;
            let (last, parent_rest) = rest.split_last()?;
            let mut new_rest = reorder_in_body_node(n, parent_rest, last, delta)?;
            let mut out = vec![Step::BodyNode(*i)];
            out.append(&mut new_rest);
            Some(out)
        }
        _ => None,
    }
}

fn reorder_in_body_node(
    n: &mut BodyNode,
    parent_rest: &[Step],
    last: &Step,
    delta: isize,
) -> Option<Vec<Step>> {
    if parent_rest.is_empty() {
        return match (n, last) {
            (BodyNode::MjWrapper(w), Step::WrapperChild(j)) => {
                swap_index(&mut w.children, *j, delta).map(|nj| vec![Step::WrapperChild(nj)])
            }
            (BodyNode::MjSection(s), Step::SectionChild(j)) => {
                swap_index(&mut s.children, *j, delta).map(|nj| vec![Step::SectionChild(nj)])
            }
            (BodyNode::MjHero(h), Step::HeroChild(j)) => {
                swap_index(&mut h.children, *j, delta).map(|nj| vec![Step::HeroChild(nj)])
            }
            _ => None,
        };
    }
    match (n, parent_rest) {
        (BodyNode::MjWrapper(w), [Step::WrapperChild(j), tail @ ..]) => {
            let child = w.children.get_mut(*j)?;
            let mut rest = reorder_in_body_node(child, tail, last, delta)?;
            let mut out = vec![Step::WrapperChild(*j)];
            out.append(&mut rest);
            Some(out)
        }
        (BodyNode::MjSection(s), [Step::SectionChild(j), tail @ ..]) => {
            match s.children.get_mut(*j)? {
                SectionChild::MjColumn(c) => {
                    let mut rest = reorder_in_column(c, tail, last, delta)?;
                    let mut out = vec![Step::SectionChild(*j)];
                    out.append(&mut rest);
                    Some(out)
                }
                SectionChild::MjGroup(g) => {
                    let mut rest = reorder_in_group(g, tail, last, delta)?;
                    let mut out = vec![Step::SectionChild(*j)];
                    out.append(&mut rest);
                    Some(out)
                }
            }
        }
        (BodyNode::MjHero(h), [Step::HeroChild(j)]) => {
            let child = h.children.get_mut(*j)?;
            let mut rest = reorder_in_column_child(child, last, delta)?;
            let mut out = vec![Step::HeroChild(*j)];
            out.append(&mut rest);
            Some(out)
        }
        _ => None,
    }
}

fn reorder_in_column(
    c: &mut MjColumn,
    parent_rest: &[Step],
    last: &Step,
    delta: isize,
) -> Option<Vec<Step>> {
    if parent_rest.is_empty() {
        return match last {
            Step::ColComp(i) => {
                swap_index(&mut c.components, *i, delta).map(|ni| vec![Step::ColComp(ni)])
            }
            _ => None,
        };
    }
    match parent_rest {
        [Step::ColComp(i)] => {
            let child = c.components.get_mut(*i)?;
            let mut rest = reorder_in_column_child(child, last, delta)?;
            let mut out = vec![Step::ColComp(*i)];
            out.append(&mut rest);
            Some(out)
        }
        _ => None,
    }
}

fn reorder_in_column_child(
    c: &mut crate::model::ColumnChild,
    last: &Step,
    delta: isize,
) -> Option<Vec<Step>> {
    match (c, last) {
        (crate::model::ColumnChild::MjNavbar(n), Step::NavbarLink(i)) => {
            swap_index(&mut n.links, *i, delta).map(|ni| vec![Step::NavbarLink(ni)])
        }
        (crate::model::ColumnChild::MjAccordion(a), Step::AccordionEl(i)) => {
            swap_index(&mut a.elements, *i, delta).map(|ni| vec![Step::AccordionEl(ni)])
        }
        (crate::model::ColumnChild::MjCarousel(car), Step::CarouselImg(i)) => {
            swap_index(&mut car.images, *i, delta).map(|ni| vec![Step::CarouselImg(ni)])
        }
        _ => None,
    }
}

fn reorder_in_group(
    g: &mut MjGroup,
    parent_rest: &[Step],
    last: &Step,
    delta: isize,
) -> Option<Vec<Step>> {
    if parent_rest.is_empty() {
        return match last {
            Step::GroupCol(i) => {
                swap_index(&mut g.children, *i, delta).map(|ni| vec![Step::GroupCol(ni)])
            }
            _ => None,
        };
    }
    match parent_rest {
        [Step::GroupCol(i), tail @ ..] => {
            let c = g.children.get_mut(*i)?;
            let mut rest = reorder_in_column(c, tail, last, delta)?;
            let mut out = vec![Step::GroupCol(*i)];
            out.append(&mut rest);
            Some(out)
        }
        _ => None,
    }
}

fn swap_index<T>(items: &mut [T], i: usize, delta: isize) -> Option<usize> {
    if items.len() < 2 {
        return None;
    }
    let j = i as isize + delta;
    if j < 0 || j >= items.len() as isize {
        return None;
    }
    let j = j as usize;
    items.swap(i, j);
    Some(j)
}

#[cfg(test)]
mod tests {
    use super::rebalance_widths;

    #[test]
    fn rebalance_examples() {
        assert_eq!(rebalance_widths(2), vec!["50%", "50%"]);
        assert_eq!(rebalance_widths(3), vec!["33%", "33%", "34%"]);
        assert_eq!(rebalance_widths(4), vec!["25%", "25%", "25%", "25%"]);
    }
}
