//! One CSS `border` value plus which sides it applies to.
//!
//! MJML accepts `border` (all sides) or `border-top` / `border-right` /
//! `border-bottom` / `border-left` with the same CSS string. Email authors
//! almost always want the same stroke on a subset of sides, so FormEdit
//! exposes checkboxes instead of four full border fields.

pub const OPTIONS: &[(&str, &str)] = &[
    ("all", "All"),
    ("top", "Top"),
    ("right", "Right"),
    ("bottom", "Bottom"),
    ("left", "Left"),
];

pub const RULE: &str = "all, or a comma/space list of top, right, bottom, left";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Sides {
    pub const ALL: Self = Self {
        top: true,
        right: true,
        bottom: true,
        left: true,
    };
    pub const NONE: Self = Self {
        top: false,
        right: false,
        bottom: false,
        left: false,
    };

    pub fn is_all(self) -> bool {
        self.top && self.right && self.bottom && self.left
    }

    pub fn is_none(self) -> bool {
        !self.top && !self.right && !self.bottom && !self.left
    }

    /// Empty / `"all"` / omitted → all four sides (backward compatible).
    pub fn from_storage(value: &str) -> Result<Self, String> {
        let t = value.trim();
        if t.is_empty() || t.eq_ignore_ascii_case("all") {
            return Ok(Self::ALL);
        }
        let mut out = Self::NONE;
        let mut saw_any = false;
        for part in t.split(|c: char| c == ',' || c.is_whitespace()) {
            let p = part.trim();
            if p.is_empty() {
                continue;
            }
            saw_any = true;
            match p.to_ascii_lowercase().as_str() {
                "all" => return Ok(Self::ALL),
                "top" => out.top = true,
                "right" => out.right = true,
                "bottom" => out.bottom = true,
                "left" => out.left = true,
                other => {
                    return Err(format!("unknown border side '{other}' (use {RULE})"));
                }
            }
        }
        if !saw_any || out.is_none() {
            return Ok(Self::ALL);
        }
        Ok(out)
    }

    /// `None` when all sides (omit from JSON). Otherwise `"top,bottom"` in TRBL order.
    pub fn to_storage(self) -> Option<String> {
        if self.is_all() || self.is_none() {
            None
        } else {
            Some(self.to_form())
        }
    }

    /// FormEdit value: `"all"` or comma-separated sides.
    pub fn to_form(self) -> String {
        if self.is_all() || self.is_none() {
            return "all".to_string();
        }
        let mut parts = Vec::new();
        if self.top {
            parts.push("top");
        }
        if self.right {
            parts.push("right");
        }
        if self.bottom {
            parts.push("bottom");
        }
        if self.left {
            parts.push("left");
        }
        parts.join(",")
    }

    pub fn has(self, option: &str) -> bool {
        match option {
            "all" => self.is_all(),
            "top" => self.top && !self.is_all(),
            "right" => self.right && !self.is_all(),
            "bottom" => self.bottom && !self.is_all(),
            "left" => self.left && !self.is_all(),
            _ => false,
        }
    }

    /// Checking All selects every side. Checking a side while All is on
    /// starts from that side only. The last remaining side cannot be
    /// unchecked (empty would silently drop the border).
    pub fn toggle(self, option: &str) -> Self {
        if option == "all" {
            return Self::ALL;
        }
        let mut next = if self.is_all() { Self::NONE } else { self };
        let before = next;
        match option {
            "top" => next.top = !next.top,
            "right" => next.right = !next.right,
            "bottom" => next.bottom = !next.bottom,
            "left" => next.left = !next.left,
            _ => return self,
        }
        if next.is_none() { before } else { next }
    }
}

/// Left+right border widths in `px` from a CSS border string, honoring sides.
/// `none` / empty / unparseable → 0. Omitted sides → all four.
pub fn horizontal_px(value: &str, sides: Option<&str>) -> f64 {
    let first = value.split_whitespace().next().unwrap_or("");
    if first.is_empty() || first.eq_ignore_ascii_case("none") || first == "0" {
        return 0.0;
    }
    let w = first
        .strip_suffix("px")
        .and_then(|n| n.parse::<f64>().ok())
        .unwrap_or(0.0);
    let sides = Sides::from_storage(sides.unwrap_or("")).unwrap_or(Sides::ALL);
    let mut total = 0.0;
    if sides.left {
        total += w;
    }
    if sides.right {
        total += w;
    }
    total
}

/// MJML attribute names for a CSS border value, given selected sides.
/// `shorthand` is `"border"` or `"inner-border"`.
pub fn mjml_attrs(shorthand: &'static str, sides: Sides) -> Vec<&'static str> {
    if sides.is_all() || sides.is_none() {
        return vec![shorthand];
    }
    let mut out = Vec::new();
    if sides.top {
        out.push(side_attr(shorthand, "top"));
    }
    if sides.right {
        out.push(side_attr(shorthand, "right"));
    }
    if sides.bottom {
        out.push(side_attr(shorthand, "bottom"));
    }
    if sides.left {
        out.push(side_attr(shorthand, "left"));
    }
    out
}

fn side_attr(shorthand: &'static str, side: &str) -> &'static str {
    match (shorthand, side) {
        ("border", "top") => "border-top",
        ("border", "right") => "border-right",
        ("border", "bottom") => "border-bottom",
        ("border", "left") => "border-left",
        ("inner-border", "top") => "inner-border-top",
        ("inner-border", "right") => "inner-border-right",
        ("inner-border", "bottom") => "inner-border-bottom",
        ("inner-border", "left") => "inner-border-left",
        _ => shorthand,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_all_are_all_sides() {
        assert_eq!(Sides::from_storage("").unwrap(), Sides::ALL);
        assert_eq!(Sides::from_storage("all").unwrap(), Sides::ALL);
        assert_eq!(Sides::from_storage("  ALL  ").unwrap(), Sides::ALL);
        assert_eq!(Sides::ALL.to_storage(), None);
        assert_eq!(Sides::ALL.to_form(), "all");
    }

    #[test]
    fn parses_subset() {
        let s = Sides::from_storage("top,bottom").unwrap();
        assert!(s.top && s.bottom && !s.left && !s.right);
        assert_eq!(s.to_form(), "top,bottom");
        assert_eq!(s.to_storage().as_deref(), Some("top,bottom"));
        let s = Sides::from_storage("left right").unwrap();
        assert_eq!(s.to_form(), "right,left");
    }

    #[test]
    fn rejects_unknown() {
        assert!(Sides::from_storage("top,foo").is_err());
    }

    #[test]
    fn toggle_all_and_sides() {
        assert_eq!(Sides::ALL.toggle("all"), Sides::ALL);
        let top = Sides::ALL.toggle("top");
        assert_eq!(top.to_form(), "top");
        let top_bottom = top.toggle("bottom");
        assert_eq!(top_bottom.to_form(), "top,bottom");
        assert_eq!(top_bottom.toggle("all").to_form(), "all");
        // last remaining side stays
        assert_eq!(top.toggle("top").to_form(), "top");
        // four individuals collapse to all
        let all = Sides::from_storage("top")
            .unwrap()
            .toggle("right")
            .toggle("bottom")
            .toggle("left");
        assert!(all.is_all());
    }

    #[test]
    fn has_shows_all_or_individuals_not_both() {
        assert!(Sides::ALL.has("all"));
        assert!(!Sides::ALL.has("top"));
        let top = Sides::ALL.toggle("top");
        assert!(!top.has("all"));
        assert!(top.has("top"));
        assert!(!top.has("bottom"));
    }

    #[test]
    fn horizontal_px_honors_sides() {
        assert_eq!(horizontal_px("1px solid #ff0000", None), 2.0);
        assert_eq!(horizontal_px("1px solid #ff0000", Some("all")), 2.0);
        assert_eq!(horizontal_px("1px solid #ff0000", Some("left")), 1.0);
        assert_eq!(horizontal_px("1px solid #ff0000", Some("top")), 0.0);
        assert_eq!(horizontal_px("none", None), 0.0);
    }

    #[test]
    fn mjml_attr_names() {
        assert_eq!(mjml_attrs("border", Sides::ALL), vec!["border"]);
        let top_bottom = Sides::from_storage("top,bottom").unwrap();
        assert_eq!(
            mjml_attrs("border", top_bottom),
            vec!["border-top", "border-bottom"]
        );
        assert_eq!(
            mjml_attrs("inner-border", top_bottom),
            vec!["inner-border-top", "inner-border-bottom"]
        );
    }
}
