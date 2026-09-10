//! MJML `padding` is a CSS-style shorthand of 1–4 `px` or `%` values.
//! Bare numbers are treated as `px` so FormEdit and emit stay in sync with MJML 5.

pub const HINT: &str = "1-4 values with px or %";
pub const PLACEHOLDER: &str = "e.g. 10px  or  10px 20px";
pub const RULE: &str = "1-4 values with px or % (e.g. 10px or 10px 20px)";
pub const UNIT_HINT: &str = "px or %";
pub const UNIT_PLACEHOLDER: &str = "e.g. 4px";
pub const UNIT_RULE: &str = "a value with px or % (e.g. 4px)";

/// Empty input is valid (optional field). Otherwise 1–4 tokens, each `0`, a
/// number + `px`/`%`, or a bare number (normalized to `px`).
/// MJML `unit(px,%){1,4}` does not accept negatives — reject them here so
/// F3 / FormEdit fail instead of a strict compile dropping the whole email.
pub fn normalize_padding(value: &str) -> Result<String, String> {
    let t = value.trim();
    if t.is_empty() {
        return Ok(String::new());
    }
    let parts: Vec<&str> = t.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 4 {
        return Err(RULE.to_string());
    }
    let mut out = Vec::with_capacity(parts.len());
    for part in parts {
        if part.starts_with('-') {
            return Err(RULE.to_string());
        }
        out.push(normalize_token(part)?);
    }
    Ok(out.join(" "))
}

/// Left+right in `px` from a 1–4 value padding shorthand.
/// `None` when empty, `%` is used, or the value is not padding.
pub fn horizontal_px(value: &str) -> Option<f64> {
    let normalized = normalize_padding(value).ok()?;
    if normalized.is_empty() {
        return None;
    }
    let parts: Vec<&str> = normalized.split_whitespace().collect();
    let px = |i: usize| -> Option<f64> {
        let t = *parts.get(i)?;
        if t == "0" {
            return Some(0.0);
        }
        t.strip_suffix("px")?.parse().ok()
    };
    match parts.len() {
        1 => Some(px(0)? * 2.0),
        2 | 3 => Some(px(1)? * 2.0),
        4 => Some(px(1)? + px(3)?),
        _ => None,
    }
}

/// Left+right border widths in `px` from a CSS border string (`1px solid #000`).
/// `none` / empty / unparseable → 0. All four sides.
pub fn border_horizontal_px(value: &str) -> f64 {
    crate::border::horizontal_px(value, None)
}

fn normalize_token(token: &str) -> Result<String, String> {
    if token == "0" {
        return Ok("0".to_string());
    }
    if let Some(num) = token.strip_suffix("px") {
        if is_unit_number(num) {
            return Ok(token.to_string());
        }
        return Err(RULE.to_string());
    }
    if let Some(num) = token.strip_suffix('%') {
        if is_unit_number(num) {
            return Ok(token.to_string());
        }
        return Err(RULE.to_string());
    }
    if is_unit_number(token) {
        return Ok(format!("{token}px"));
    }
    Err(RULE.to_string())
}

/// One `px`/`%` value, or `0`. Bare numbers become `px`. `auto` is allowed for
/// heights.
pub fn normalize_unit(value: &str) -> Result<String, String> {
    let t = value.trim();
    if t.is_empty() {
        return Ok(String::new());
    }
    if t.eq_ignore_ascii_case("auto") {
        return Ok("auto".to_string());
    }
    normalize_token(t).map_err(|_| UNIT_RULE.to_string())
}

fn is_unit_number(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for c in s.chars() {
        if c == '.' {
            if seen_dot {
                return false;
            }
            seen_dot = true;
        } else if c.is_ascii_digit() {
            seen_digit = true;
        } else {
            return false;
        }
    }
    seen_digit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_ok() {
        assert_eq!(normalize_padding("").unwrap(), "");
        assert_eq!(normalize_padding("   ").unwrap(), "");
    }

    #[test]
    fn units_pass_through() {
        assert_eq!(normalize_padding("10px").unwrap(), "10px");
        assert_eq!(normalize_padding("10%").unwrap(), "10%");
        assert_eq!(normalize_padding("10px 20px").unwrap(), "10px 20px");
        assert_eq!(
            normalize_padding("12px 10px 12px 10px").unwrap(),
            "12px 10px 12px 10px"
        );
        assert_eq!(normalize_padding("0").unwrap(), "0");
        assert_eq!(normalize_padding("0 10px").unwrap(), "0 10px");
        assert_eq!(normalize_padding("1.5px").unwrap(), "1.5px");
    }

    #[test]
    fn bare_numbers_become_px() {
        assert_eq!(normalize_padding("2").unwrap(), "2px");
        assert_eq!(
            normalize_padding("12 10 12 10").unwrap(),
            "12px 10px 12px 10px"
        );
        assert_eq!(normalize_padding("10px 20").unwrap(), "10px 20px");
    }

    #[test]
    fn rejects_bad_values() {
        assert!(normalize_padding("10em").is_err());
        assert!(normalize_padding("foo").is_err());
        assert!(normalize_padding("1 2 3 4 5").is_err());
        assert!(normalize_padding("10 px").is_err());
        assert!(normalize_padding("px").is_err());
        assert!(normalize_padding("10PX").is_err());
        assert!(normalize_padding("-2px").is_err());
        assert!(normalize_padding("10px -4px").is_err());
    }

    #[test]
    fn horizontal_px_from_shorthand() {
        assert_eq!(horizontal_px("10px").unwrap(), 20.0);
        assert_eq!(horizontal_px("12px 24px").unwrap(), 48.0);
        assert_eq!(horizontal_px("1px 2px 3px 4px").unwrap(), 6.0);
        assert!(horizontal_px("10%").is_none());
        assert!(horizontal_px("").is_none());
        assert_eq!(border_horizontal_px("1px solid #ff0000"), 2.0);
        assert_eq!(border_horizontal_px("none"), 0.0);
    }

    #[test]
    fn single_unit() {
        assert_eq!(normalize_unit("4").unwrap(), "4px");
        assert_eq!(normalize_unit("4px").unwrap(), "4px");
        assert_eq!(normalize_unit("auto").unwrap(), "auto");
        assert!(normalize_unit("4em").is_err());
    }
}
