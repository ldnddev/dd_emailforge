//! Column width bookkeeping for adding and removing columns.
//!
//! Adding a column never throws away custom widths: when the existing columns
//! still use the default equal split they stay equal, otherwise the new column
//! gets `100 / (n + 1)` percent and the others shrink in proportion. Removing a
//! column hands its share back to the remaining columns in proportion. Results
//! are rounded to hundredths of a percent and always total exactly 100%.
//! Widths that are not percentages (for example `px`) are left untouched.
use crate::model::{MjColumn, MjGroup, MjSection, SectionChild};

use super::edits::rebalance_widths;

/// Hundredths of a percent in a full row.
const FULL: u32 = 10_000;

fn parse_pct(w: &str) -> Option<f64> {
    let v: f64 = w.trim().strip_suffix('%')?.trim().parse().ok()?;
    (v.is_finite() && v > 0.0).then_some(v)
}

fn format_pct(hundredths: u32) -> String {
    if hundredths.is_multiple_of(100) {
        format!("{}%", hundredths / 100)
    } else {
        let s = format!("{}.{:02}", hundredths / 100, hundredths % 100);
        format!("{}%", s.trim_end_matches('0'))
    }
}

/// True when the widths are still the editor's default equal split (unset,
/// the integer split `rebalance_widths` produces, or all-equal percentages).
fn is_default_split(widths: &[Option<String>]) -> bool {
    if widths.iter().all(Option::is_none) {
        return true;
    }
    let defaults = rebalance_widths(widths.len());
    if widths
        .iter()
        .zip(&defaults)
        .all(|(w, d)| w.as_deref() == Some(d.as_str()))
    {
        return true;
    }
    let Some(vals) = widths
        .iter()
        .map(|w| w.as_deref().and_then(parse_pct))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let sum: f64 = vals.iter().sum();
    (sum - 100.0).abs() < 0.5 && vals.iter().all(|v| (v - vals[0]).abs() < 0.01)
}

/// Percent values for every column, filling unset widths with an equal share
/// of whatever the set ones leave. `None` if any width is not a percentage or
/// nothing is left for the unset ones.
fn resolve_pcts(widths: &[Option<String>]) -> Option<Vec<f64>> {
    let mut known = 0.0;
    let mut unset = 0usize;
    for w in widths {
        match w.as_deref() {
            None | Some("") => unset += 1,
            Some(s) => known += parse_pct(s)?,
        }
    }
    let fill = if unset > 0 {
        let left = 100.0 - known;
        if left <= 0.0 {
            return None;
        }
        left / unset as f64
    } else {
        0.0
    };
    Some(
        widths
            .iter()
            .map(|w| match w.as_deref() {
                None | Some("") => fill,
                Some(s) => parse_pct(s).unwrap_or(fill),
            })
            .collect(),
    )
}

/// Scale `vals` to hundredths totalling exactly `FULL` (largest remainder).
/// `None` if a column would round down to 0%.
fn to_hundredths(vals: &[f64]) -> Option<Vec<u32>> {
    let sum: f64 = vals.iter().sum();
    if sum <= 0.0 {
        return None;
    }
    let exact: Vec<f64> = vals.iter().map(|v| v / sum * FULL as f64).collect();
    let mut out: Vec<u32> = exact.iter().map(|x| x.floor() as u32).collect();
    let mut short = FULL - out.iter().sum::<u32>();
    let mut order: Vec<usize> = (0..vals.len()).collect();
    order.sort_by(|&a, &b| {
        let fa = exact[a] - exact[a].floor();
        let fb = exact[b] - exact[b].floor();
        fb.partial_cmp(&fa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    for i in order {
        if short == 0 {
            break;
        }
        out[i] += 1;
        short -= 1;
    }
    out.iter().all(|&h| h > 0).then_some(out)
}

/// Widths for a column list after inserting a new column at `at`.
pub(super) fn widths_after_insert(existing: &[Option<String>], at: usize) -> Vec<Option<String>> {
    let n = existing.len();
    let at = at.min(n);
    if is_default_split(existing) {
        return rebalance_widths(n + 1).into_iter().map(Some).collect();
    }
    let Some(vals) = resolve_pcts(existing) else {
        // Non-percent widths (px): keep them; the new column takes no width
        // and MJML sizes it from what is left.
        let mut out = existing.to_vec();
        out.insert(at, None);
        return out;
    };
    let new_share = 100.0 / (n + 1) as f64;
    let sum: f64 = vals.iter().sum();
    let mut scaled: Vec<f64> = vals.iter().map(|v| v / sum * (100.0 - new_share)).collect();
    scaled.insert(at, new_share);
    match to_hundredths(&scaled) {
        Some(h) => h.into_iter().map(|h| Some(format_pct(h))).collect(),
        None => rebalance_widths(n + 1).into_iter().map(Some).collect(),
    }
}

/// Widths for a column list after removing the column at `removed`.
pub(super) fn widths_after_remove(
    existing: &[Option<String>],
    removed: usize,
) -> Vec<Option<String>> {
    let mut rest = existing.to_vec();
    if removed < rest.len() {
        rest.remove(removed);
    }
    if rest.is_empty() {
        return rest;
    }
    if is_default_split(existing) {
        return rebalance_widths(rest.len()).into_iter().map(Some).collect();
    }
    let Some(vals) = resolve_pcts(existing) else {
        return rest;
    };
    let mut vals = vals;
    vals.remove(removed.min(vals.len() - 1));
    match to_hundredths(&vals) {
        Some(h) => h.into_iter().map(|h| Some(format_pct(h))).collect(),
        None => rebalance_widths(rest.len()).into_iter().map(Some).collect(),
    }
}

fn section_widths(s: &MjSection) -> Vec<Option<String>> {
    s.children
        .iter()
        .filter_map(|c| match c {
            SectionChild::MjColumn(col) => Some(col.width.clone()),
            _ => None,
        })
        .collect()
}

fn set_section_widths(s: &mut MjSection, widths: Vec<Option<String>>) {
    let mut it = widths.into_iter();
    for child in &mut s.children {
        if let SectionChild::MjColumn(c) = child {
            c.width = it.next().flatten();
        }
    }
}

/// Insert `col` into the section at child index `at`, resizing the columns.
/// Returns the child index the column landed at.
pub(super) fn insert_section_column(s: &mut MjSection, at: usize, col: MjColumn) -> usize {
    let at = at.min(s.children.len());
    let ordinal = s.children[..at]
        .iter()
        .filter(|c| matches!(c, SectionChild::MjColumn(_)))
        .count();
    let widths = widths_after_insert(&section_widths(s), ordinal);
    s.children.insert(at, SectionChild::MjColumn(col));
    set_section_widths(s, widths);
    at
}

/// Remove the section child at `idx` (a column), resizing the rest.
pub(super) fn remove_section_column(s: &mut MjSection, idx: usize) {
    if idx >= s.children.len() {
        return;
    }
    let is_col = matches!(s.children[idx], SectionChild::MjColumn(_));
    let ordinal = s.children[..idx]
        .iter()
        .filter(|c| matches!(c, SectionChild::MjColumn(_)))
        .count();
    let before = section_widths(s);
    s.children.remove(idx);
    if is_col {
        set_section_widths(s, widths_after_remove(&before, ordinal));
    }
}

pub(super) fn insert_group_column(g: &mut MjGroup, at: usize, col: MjColumn) -> usize {
    let at = at.min(g.children.len());
    let before: Vec<_> = g.children.iter().map(|c| c.width.clone()).collect();
    let widths = widths_after_insert(&before, at);
    g.children.insert(at, col);
    for (c, w) in g.children.iter_mut().zip(widths) {
        c.width = w;
    }
    at
}

pub(super) fn remove_group_column(g: &mut MjGroup, idx: usize) {
    if idx >= g.children.len() {
        return;
    }
    let before: Vec<_> = g.children.iter().map(|c| c.width.clone()).collect();
    g.children.remove(idx);
    for (c, w) in g.children.iter_mut().zip(widths_after_remove(&before, idx)) {
        c.width = w;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(v: &[&str]) -> Vec<Option<String>> {
        v.iter().map(|s| Some((*s).to_string())).collect()
    }
    fn total(ws: &[Option<String>]) -> u32 {
        ws.iter()
            .map(|w| (parse_pct(w.as_deref().unwrap()).unwrap() * 100.0).round() as u32)
            .sum()
    }

    #[test]
    fn insert_into_custom_30_70_scales_proportionally() {
        let out = widths_after_insert(&w(&["30%", "70%"]), 2);
        assert_eq!(out, w(&["20%", "46.67%", "33.33%"]));
        assert_eq!(total(&out), 10_000);
    }

    #[test]
    fn insert_in_the_middle_of_custom_widths() {
        let out = widths_after_insert(&w(&["30%", "70%"]), 1);
        assert_eq!(out, w(&["20%", "33.33%", "46.67%"]));
        assert_eq!(total(&out), 10_000);
    }

    #[test]
    fn insert_keeps_default_equal_split() {
        assert_eq!(widths_after_insert(&w(&["100%"]), 1), w(&["50%", "50%"]));
        assert_eq!(
            widths_after_insert(&w(&["50%", "50%"]), 2),
            w(&["33%", "33%", "34%"])
        );
        assert_eq!(
            widths_after_insert(&w(&["33%", "33%", "34%"]), 3),
            w(&["25%", "25%", "25%", "25%"])
        );
        assert_eq!(
            widths_after_insert(&[None, None], 2),
            w(&["33%", "33%", "34%"])
        );
        assert_eq!(widths_after_insert(&[], 0), w(&["100%"]));
    }

    #[test]
    fn insert_into_uneven_three_columns_totals_100() {
        let out = widths_after_insert(&w(&["25%", "25%", "50%"]), 3);
        assert_eq!(out, w(&["18.75%", "18.75%", "37.5%", "25%"]));
        let out = widths_after_insert(&w(&["10%", "20%", "70%"]), 0);
        assert_eq!(total(&out), 10_000);
        assert_eq!(out[0], Some("25%".into()));
    }

    #[test]
    fn insert_fills_unset_widths_from_the_remainder() {
        // 60% set, the unset column takes the remaining 40%.
        let out = widths_after_insert(&[Some("60%".into()), None], 2);
        assert_eq!(out, w(&["40%", "26.67%", "33.33%"]));
    }

    #[test]
    fn insert_leaves_px_widths_alone() {
        let out = widths_after_insert(&w(&["200px", "400px"]), 2);
        assert_eq!(out, vec![Some("200px".into()), Some("400px".into()), None]);
    }

    #[test]
    fn remove_redistributes_proportionally() {
        assert_eq!(
            widths_after_remove(&w(&["20%", "46.67%", "33.33%"]), 2),
            w(&["30%", "70%"])
        );
        let out = widths_after_remove(&w(&["25%", "25%", "50%"]), 0);
        assert_eq!(out, w(&["33.33%", "66.67%"]));
        assert_eq!(total(&out), 10_000);
    }

    #[test]
    fn remove_keeps_default_equal_split() {
        assert_eq!(
            widths_after_remove(&w(&["33%", "33%", "34%"]), 1),
            w(&["50%", "50%"])
        );
        assert_eq!(widths_after_remove(&w(&["50%", "50%"]), 0), w(&["100%"]));
    }

    #[test]
    fn remove_leaves_px_widths_alone() {
        assert_eq!(
            widths_after_remove(&w(&["200px", "400px", "100px"]), 1),
            w(&["200px", "100px"])
        );
    }

    #[test]
    fn section_helpers_skip_groups() {
        let mut s = MjSection::default();
        for x in ["30%", "70%"] {
            s.children.push(SectionChild::MjColumn(MjColumn {
                width: Some(x.into()),
                ..Default::default()
            }));
        }
        s.children.push(SectionChild::MjGroup(MjGroup::default()));
        let at = insert_section_column(&mut s, 2, MjColumn::default());
        assert_eq!(at, 2);
        assert_eq!(section_widths(&s), w(&["20%", "46.67%", "33.33%"]));
        remove_section_column(&mut s, 2);
        assert_eq!(section_widths(&s), w(&["30%", "70%"]));
    }
}
