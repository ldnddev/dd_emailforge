//! Inspector fields plus a full-email ascii blueprint. Click a region to select it.
use crate::model::{
    BodyNode, ColumnChild, EmailArticle, EmailFooter, EmailHeader, EmailHero, ImagePosition,
    MjColumn, MjGroup, MjHero, MjSection, MjWrapper, SectionChild, Template,
};

use super::tree::{Step, TreeId, TreeRow};

pub fn details_title(label: &str) -> String {
    format!("Details — {label}")
}

#[derive(Clone, Debug)]
pub struct DetailHit {
    pub line: usize,
    pub x0: usize,
    pub x1: usize,
    pub id: TreeId,
}

#[cfg(test)]
pub fn details_lines(
    template: Option<&Template>,
    row: Option<&TreeRow>,
    width: usize,
) -> Vec<String> {
    details_view(template, row, width).0
}

pub fn details_view(
    template: Option<&Template>,
    row: Option<&TreeRow>,
    width: usize,
) -> (Vec<String>, Vec<DetailHit>) {
    let Some(t) = template else {
        return (vec!["No template open.".into()], Vec::new());
    };
    let Some(row) = row else {
        return (vec!["Nothing selected.".into()], Vec::new());
    };
    let mut lines = match &row.id {
        TreeId::Head => head_lines(t),
        TreeId::Brand => brand_lines(t),
        TreeId::Body => vec![format!("nodes: {}", t.body.nodes.len())],
        TreeId::Path(path) => selection_summary(t, path),
    };
    if !lines.is_empty() {
        lines.push(String::new());
    }
    let (ascii, mut hits) = email_blueprint(t, width);
    let offset = lines.len();
    for h in &mut hits {
        h.line += offset;
    }
    lines.extend(ascii);
    (lines, hits)
}

fn head_lines(t: &Template) -> Vec<String> {
    let fonts = if t.head.fonts.is_empty() {
        "(none)".into()
    } else {
        t.head
            .fonts
            .iter()
            .map(|f| format!("{} ({})", f.name, host_of(&f.href)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let json_ld = json_ld_summary(&t.head.json_ld);
    let css = if t.head.css.trim().is_empty() {
        "(empty)".into()
    } else {
        format!("{} lines", t.head.css.lines().count())
    };
    vec![
        format!("subject:     {}", dash(&t.subject)),
        format!("preheader:   {}", dash(&t.preheader)),
        format!("lang:        {}", dash(&t.lang)),
        format!("dir:         {}", dash(&t.dir)),
        format!("title:       {}", dash(&t.head.title)),
        format!("breakpoint:  {}", dash(&t.head.breakpoint)),
        format!("base_url:    {}", dash(&t.base_url)),
        format!("fonts:       {fonts}"),
        format!("json_ld:     {json_ld}"),
        format!("css:         {css}"),
        format!("css_inline:  {}", t.head.css_inline),
    ]
}

fn brand_lines(t: &Template) -> Vec<String> {
    vec![
        format!("font_family:         {}", t.brand.font_family),
        format!("text_color:          {}", t.brand.text_color),
        format!("background_color:    {}", t.brand.background_color),
        format!("content_width:       {}", t.brand.content_width),
        format!("button_background:   {}", t.brand.button_background),
        format!("button_color:        {}", t.brand.button_color),
    ]
}

fn selection_summary(t: &Template, path: &[Step]) -> Vec<String> {
    match locate(t, path) {
        Located::BodyNode(n) => body_node_summary(n),
        Located::Column(c) => vec![
            format!("width: {}", c.width.as_deref().unwrap_or("(equal split)")),
            format!("components: {}", c.components.len()),
        ],
        Located::Leaf(label, extra) => {
            let mut v = vec![label];
            v.extend(extra);
            v
        }
        Located::Missing => vec!["(missing node)".into()],
    }
}

enum Located<'a> {
    BodyNode(&'a BodyNode),
    Column(&'a MjColumn),
    Leaf(String, Vec<String>),
    Missing,
}

fn locate<'a>(t: &'a Template, path: &[Step]) -> Located<'a> {
    let mut steps = path.iter();
    let Some(Step::BodyNode(i)) = steps.next() else {
        return Located::Missing;
    };
    let mut node = match t.body.nodes.get(*i) {
        Some(n) => n,
        None => return Located::Missing,
    };
    loop {
        match steps.next() {
            None => return Located::BodyNode(node),
            Some(Step::WrapperChild(j)) => {
                let BodyNode::MjWrapper(w) = node else {
                    return Located::Missing;
                };
                node = match w.children.get(*j) {
                    Some(n) => n,
                    None => return Located::Missing,
                };
            }
            Some(Step::SectionChild(j)) => {
                let BodyNode::MjSection(s) = node else {
                    return Located::Missing;
                };
                match s.children.get(*j) {
                    Some(SectionChild::MjColumn(c)) => {
                        return locate_column(c, steps);
                    }
                    Some(SectionChild::MjGroup(g)) => {
                        return locate_group(g, steps);
                    }
                    None => return Located::Missing,
                }
            }
            Some(Step::HeroChild(j)) => {
                let BodyNode::MjHero(h) = node else {
                    return Located::Missing;
                };
                return match h.children.get(*j) {
                    Some(c) => locate_nested_child(c, steps),
                    None => Located::Missing,
                };
            }
            _ => return Located::Missing,
        }
    }
}

fn locate_group<'a>(g: &'a MjGroup, mut steps: std::slice::Iter<'_, Step>) -> Located<'a> {
    match steps.next() {
        None => Located::Leaf(
            "mj-group".into(),
            vec![format!("columns: {}", g.children.len())],
        ),
        Some(Step::GroupCol(i)) => match g.children.get(*i) {
            Some(c) => locate_column(c, steps),
            None => Located::Missing,
        },
        _ => Located::Missing,
    }
}

fn locate_column<'a>(c: &'a MjColumn, mut steps: std::slice::Iter<'_, Step>) -> Located<'a> {
    match steps.next() {
        None => Located::Column(c),
        Some(Step::ColComp(i)) => match c.components.get(*i) {
            Some(ch) => locate_nested_child(ch, steps),
            None => Located::Missing,
        },
        _ => Located::Missing,
    }
}

fn locate_nested_child<'a>(
    ch: &'a ColumnChild,
    mut steps: std::slice::Iter<'_, Step>,
) -> Located<'a> {
    match steps.next() {
        None => {
            let extra = match ch {
                ColumnChild::MjNavbar(n) => vec![format!("links: {}", n.links.len())],
                ColumnChild::MjAccordion(a) => vec![format!("elements: {}", a.elements.len())],
                ColumnChild::MjCarousel(c) => vec![format!("images: {}", c.images.len())],
                _ => vec![],
            };
            Located::Leaf(column_child_detail(ch), extra)
        }
        Some(Step::NavbarLink(i)) => match ch {
            ColumnChild::MjNavbar(n) => match n.links.get(*i) {
                Some(l) => Located::Leaf(
                    format!("mj-navbar-link  {} → {}", l.content, l.href),
                    vec![],
                ),
                None => Located::Missing,
            },
            _ => Located::Missing,
        },
        Some(Step::AccordionEl(i)) => match ch {
            ColumnChild::MjAccordion(a) => match a.elements.get(*i) {
                Some(el) => Located::Leaf(
                    format!("mj-accordion-element  {}", truncate(&el.title, 40)),
                    vec![],
                ),
                None => Located::Missing,
            },
            _ => Located::Missing,
        },
        Some(Step::CarouselImg(i)) => match ch {
            ColumnChild::MjCarousel(c) => match c.images.get(*i) {
                Some(img) => Located::Leaf(format!("mj-carousel-image  {}", img.src), vec![]),
                None => Located::Missing,
            },
            _ => Located::Missing,
        },
        _ => Located::Missing,
    }
}

fn body_node_summary(n: &BodyNode) -> Vec<String> {
    match n {
        BodyNode::MjSection(s) => {
            let mut lines = Vec::new();
            if let Some(label) = s.label.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                lines.push(format!("label: {label}"));
            }
            lines.push(format!("children: {}", s.children.len()));
            if let Some(bg) = &s.background_color {
                lines.push(format!("background: {bg}"));
            }
            lines
        }
        BodyNode::MjWrapper(w) => {
            let mut lines = Vec::new();
            if let Some(label) = w.label.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                lines.push(format!("label: {label}"));
            }
            lines.push(format!("wrapper children: {}", w.children.len()));
            lines.push(format!("full_width: {}", w.full_width));
            lines
        }
        BodyNode::MjHero(h) => {
            let mut lines = vec![format!("hero children: {}", h.children.len())];
            if let Some(url) = &h.background_url {
                lines.push(format!("background_url: {url}"));
            }
            lines
        }
        BodyNode::EmailHeader(h) => email_header_lines(h),
        BodyNode::EmailHero(h) => email_hero_lines(h),
        BodyNode::EmailCta(c) => vec![
            format!("heading: {}", dash(&c.heading)),
            format!("copy:    {}", dash(&c.copy)),
            format!(
                "button:  {} → {}",
                dash(&c.button_label),
                dash(&c.button_href)
            ),
        ],
        BodyNode::EmailArticle(a) => email_article_lines(a),
        BodyNode::EmailFooter(f) => email_footer_lines(f),
    }
}

fn email_header_lines(h: &EmailHeader) -> Vec<String> {
    vec![
        format!("logo_src: {}", dash(&h.logo_src)),
        format!("logo_alt: {}", dash(&h.logo_alt)),
        format!("logo_width: {}", h.logo_width),
    ]
}

fn email_hero_lines(h: &EmailHero) -> Vec<String> {
    vec![
        format!("heading:    {}", dash(&h.heading)),
        format!("subheading: {}", dash(&h.subheading)),
        format!("image_src:  {}", dash(&h.image_src)),
    ]
}

fn email_article_lines(a: &EmailArticle) -> Vec<String> {
    vec![
        format!("title:     {}", dash(&a.title)),
        format!("copy:      {}", dash(&a.copy)),
        format!("image:     {}", dash(&a.image_src)),
        format!("position:  {:?}", a.image_position),
    ]
}

fn email_footer_lines(f: &EmailFooter) -> Vec<String> {
    vec![
        format!("company: {}", dash(&f.company_name)),
        format!("address: {}", f.address_lines.join(" / ")),
        format!("unsub:   {}", dash(&f.unsubscribe_href)),
    ]
}

fn column_child_detail(c: &ColumnChild) -> String {
    match c {
        ColumnChild::MjText(t) => format!("mj-text  {}", truncate(&t.content, 40)),
        ColumnChild::MjButton(b) => format!("mj-button  {} → {}", b.content, b.href),
        ColumnChild::MjImage(i) => format!("mj-image  {}", i.src),
        ColumnChild::MjDivider(_) => "mj-divider".into(),
        ColumnChild::MjSpacer(s) => format!("mj-spacer  {}", s.height),
        ColumnChild::MjSocial(s) => format!("mj-social  {} icons", s.elements.len()),
        ColumnChild::MjTable(_) => "mj-table".into(),
        ColumnChild::MjNavbar(n) => format!("mj-navbar  {} links", n.links.len()),
        ColumnChild::MjAccordion(a) => format!("mj-accordion  {} items", a.elements.len()),
        ColumnChild::MjCarousel(c) => format!("mj-carousel  {} images", c.images.len()),
    }
}

fn email_blueprint(t: &Template, width: usize) -> (Vec<String>, Vec<DetailHit>) {
    let mut c = Canvas::new(width.max(12));
    c.fill_line(
        0,
        0,
        c.width,
        &format!("{}px canvas", t.brand.content_width),
        None,
    );
    let mut y = 1;
    let width = c.width;
    if t.body.nodes.is_empty() {
        let mut inner = Canvas::new(width.saturating_sub(2).max(1));
        let inner_w = inner.width;
        inner.fill_line(0, 0, inner_w, "(empty body)", Some(&TreeId::Body));
        y = blit_boxed(&mut c, 0, y, width, &TreeId::Body, inner);
    } else {
        for (i, node) in t.body.nodes.iter().enumerate() {
            y = paint_body_node(&mut c, node, vec![Step::BodyNode(i)], 0, y, width);
        }
    }
    let _ = y;
    c.finish()
}

fn paint_body_node(
    c: &mut Canvas,
    node: &BodyNode,
    path: Vec<Step>,
    x: usize,
    y: usize,
    w: usize,
) -> usize {
    let id = TreeId::Path(path.clone());
    match node {
        BodyNode::MjSection(s) => {
            let inner = paint_section_inner(s, &path, w.saturating_sub(2).max(1));
            blit_boxed(c, x, y, w, &id, inner)
        }
        BodyNode::MjWrapper(wrapper) => {
            let inner = paint_wrapper_inner(wrapper, &path, w.saturating_sub(2).max(1));
            blit_boxed(c, x, y, w, &id, inner)
        }
        BodyNode::MjHero(h) => {
            let inner = paint_hero_inner(h, &path, w.saturating_sub(2).max(1));
            blit_boxed(c, x, y, w, &id, inner)
        }
        BodyNode::EmailHeader(h) => paint_labeled_box(
            c,
            x,
            y,
            w,
            &id,
            "email-header",
            &[
                format!("logo {}", dash(&h.logo_src)),
                format!("{}", dash(&h.logo_alt)),
            ],
        ),
        BodyNode::EmailHero(h) => paint_labeled_box(
            c,
            x,
            y,
            w,
            &id,
            "email-hero",
            &[
                format!("{}", dash(&h.heading)),
                format!("{}", dash(&h.subheading)),
            ],
        ),
        BodyNode::EmailCta(n) => paint_labeled_box(
            c,
            x,
            y,
            w,
            &id,
            "email-cta",
            &[
                format!("{}", dash(&n.heading)),
                format!("btn {}", dash(&n.button_label)),
            ],
        ),
        BodyNode::EmailArticle(a) => paint_article_box(c, x, y, w, &id, a),
        BodyNode::EmailFooter(f) => paint_labeled_box(
            c,
            x,
            y,
            w,
            &id,
            "email-footer",
            &[
                format!("{}", dash(&f.company_name)),
                format!("{}", dash(&f.unsubscribe_label)),
            ],
        ),
    }
}

fn paint_section_inner(s: &MjSection, path: &[Step], w: usize) -> Canvas {
    let mut inner = Canvas::new(w);
    let mut y = 0;
    if let Some(label) = s.label.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let id = TreeId::Path(path.to_vec());
        inner.fill_line(0, 0, w, label, Some(&id));
        y = 1;
    }
    if s.children.is_empty() {
        inner.fill_line(0, y, w, "(no columns)", None);
        return inner;
    }
    paint_section_slots(&mut inner, &s.children, path, w, y);
    inner
}

fn paint_wrapper_inner(wpr: &MjWrapper, path: &[Step], w: usize) -> Canvas {
    let mut inner = Canvas::new(w);
    let caption = match wpr
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(l) => format!("mj-wrapper  {l}"),
        None => "mj-wrapper".into(),
    };
    inner.fill_line(0, 0, w, &caption, None);
    let mut y = 1;
    if wpr.children.is_empty() {
        inner.fill_line(0, y, w, "(empty)", None);
    } else {
        for (i, child) in wpr.children.iter().enumerate() {
            let mut p = path.to_vec();
            p.push(Step::WrapperChild(i));
            y = paint_body_node(&mut inner, child, p, 0, y, w);
        }
    }
    inner
}

fn paint_hero_inner(h: &MjHero, path: &[Step], w: usize) -> Canvas {
    let mut inner = Canvas::new(w);
    inner.fill_line(0, 0, w, "mj-hero", None);
    let mut y = 1;
    if h.children.is_empty() {
        inner.fill_line(0, y, w, "(empty)", None);
    } else {
        for (i, ch) in h.children.iter().enumerate() {
            let mut p = path.to_vec();
            p.push(Step::HeroChild(i));
            y = paint_component(&mut inner, ch, p, y, w);
        }
    }
    inner
}

fn paint_labeled_box(
    c: &mut Canvas,
    x: usize,
    y: usize,
    w: usize,
    id: &TreeId,
    title: &str,
    extra: &[String],
) -> usize {
    let iw = w.saturating_sub(2).max(1);
    let mut inner = Canvas::new(iw);
    inner.fill_line(0, 0, iw, title, Some(id));
    for (i, line) in extra.iter().enumerate() {
        if line.trim().is_empty() || line.trim() == "—" {
            continue;
        }
        inner.fill_line(0, i + 1, iw, line, Some(id));
    }
    blit_boxed(c, x, y, w, id, inner)
}

fn paint_article_box(
    c: &mut Canvas,
    x: usize,
    y: usize,
    w: usize,
    id: &TreeId,
    a: &EmailArticle,
) -> usize {
    let pos = match a.image_position {
        ImagePosition::Top => "image top",
        ImagePosition::Left => "image left",
        ImagePosition::Right => "image right",
    };
    paint_labeled_box(
        c,
        x,
        y,
        w,
        id,
        "email-article",
        &[format!("{}", dash(&a.title)), pos.to_string()],
    )
}

fn paint_section_slots(
    c: &mut Canvas,
    children: &[SectionChild],
    path: &[Step],
    w: usize,
    y: usize,
) {
    let n = children.len();
    if n == 0 {
        return;
    }
    let widths = distribute_widths(
        w,
        n,
        children.iter().map(|ch| match ch {
            SectionChild::MjColumn(col) => col.width.as_deref(),
            SectionChild::MjGroup(g) => g.width.as_deref(),
        }),
    );
    let mut parts = Vec::new();
    for (i, ch) in children.iter().enumerate() {
        let mut p = path.to_vec();
        p.push(Step::SectionChild(i));
        parts.push(paint_slot(ch, p, widths[i]));
    }
    blit_row(c, &parts, &widths, y);
}

fn paint_slot(ch: &SectionChild, path: Vec<Step>, w: usize) -> Canvas {
    match ch {
        SectionChild::MjColumn(col) => paint_column(col, path, w),
        SectionChild::MjGroup(g) => paint_group(g, path, w),
    }
}

fn paint_column(col: &MjColumn, path: Vec<Step>, w: usize) -> Canvas {
    let id = TreeId::Path(path.clone());
    let mut c = Canvas::new(w.max(1));
    let label = match col
        .width
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(width) => format!("mj-column {width}"),
        None => "mj-column".into(),
    };
    c.fill_line(0, 0, w, &label, Some(&id));
    let mut y = 1;
    if col.components.is_empty() {
        c.fill_line(0, y, w, "(empty)", Some(&id));
    } else {
        for (i, ch) in col.components.iter().enumerate() {
            let mut p = path.clone();
            p.push(Step::ColComp(i));
            y = paint_component(&mut c, ch, p, y, w);
        }
    }
    c.fill_missing(&id);
    c
}

fn paint_group(g: &MjGroup, path: Vec<Step>, w: usize) -> Canvas {
    let id = TreeId::Path(path.clone());
    let mut c = Canvas::new(w.max(1));
    c.fill_line(0, 0, w, "mj-group", Some(&id));
    if g.children.is_empty() {
        c.fill_line(0, 1, w, "(no columns)", Some(&id));
        c.fill_missing(&id);
        return c;
    }
    let n = g.children.len();
    let widths = distribute_widths(w, n, g.children.iter().map(|col| col.width.as_deref()));
    let mut parts = Vec::new();
    for (i, col) in g.children.iter().enumerate() {
        let mut p = path.clone();
        p.push(Step::GroupCol(i));
        parts.push(paint_column(col, p, widths[i]));
    }
    let mut row = Canvas::new(w.max(1));
    blit_row(&mut row, &parts, &widths, 0);
    c.blit(&row, 0, 1);
    c.fill_missing(&id);
    c
}

fn paint_component(c: &mut Canvas, ch: &ColumnChild, path: Vec<Step>, y: usize, w: usize) -> usize {
    let id = TreeId::Path(path.clone());
    match ch {
        ColumnChild::MjNavbar(n) => {
            c.fill_line(0, y, w, &column_child_detail(ch), Some(&id));
            let mut yy = y + 1;
            for (i, link) in n.links.iter().enumerate() {
                let mut p = path.clone();
                p.push(Step::NavbarLink(i));
                let lid = TreeId::Path(p);
                c.fill_line(
                    0,
                    yy,
                    w,
                    &format!("  {}", truncate(&link.content, w.saturating_sub(3))),
                    Some(&lid),
                );
                yy += 1;
            }
            yy
        }
        ColumnChild::MjAccordion(a) => {
            c.fill_line(0, y, w, &column_child_detail(ch), Some(&id));
            let mut yy = y + 1;
            for (i, el) in a.elements.iter().enumerate() {
                let mut p = path.clone();
                p.push(Step::AccordionEl(i));
                let eid = TreeId::Path(p);
                c.fill_line(
                    0,
                    yy,
                    w,
                    &format!("  {}", truncate(&el.title, w.saturating_sub(3))),
                    Some(&eid),
                );
                yy += 1;
            }
            yy
        }
        ColumnChild::MjCarousel(car) => {
            c.fill_line(0, y, w, &column_child_detail(ch), Some(&id));
            let mut yy = y + 1;
            for (i, img) in car.images.iter().enumerate() {
                let mut p = path.clone();
                p.push(Step::CarouselImg(i));
                let iid = TreeId::Path(p);
                let label = if img.alt.trim().is_empty() {
                    truncate(&img.src, w.saturating_sub(3))
                } else {
                    truncate(&img.alt, w.saturating_sub(3))
                };
                c.fill_line(0, yy, w, &format!("  {label}"), Some(&iid));
                yy += 1;
            }
            yy
        }
        other => {
            c.fill_line(0, y, w, &column_child_detail(other), Some(&id));
            y + 1
        }
    }
}

fn blit_boxed(
    c: &mut Canvas,
    x: usize,
    y: usize,
    w: usize,
    id: &TreeId,
    mut inner: Canvas,
) -> usize {
    inner.fill_missing(id);
    let ih = inner.height().max(1);
    let h = ih + 2;
    c.rect(x, y, w, h, Some(id));
    c.blit(&inner, x.saturating_add(1), y.saturating_add(1));
    y + h
}

fn blit_row(dst: &mut Canvas, parts: &[Canvas], widths: &[usize], y: usize) {
    let h = parts.iter().map(|p| p.height().max(1)).max().unwrap_or(1);
    let mut x = 0;
    for (i, part) in parts.iter().enumerate() {
        dst.blit(part, x, y);
        x += widths.get(i).copied().unwrap_or(part.width);
        if i + 1 < parts.len() {
            for yy in y..y + h {
                dst.set(x, yy, '|', None);
            }
            x += 1;
        }
    }
}

fn distribute_widths<'a>(
    total: usize,
    n: usize,
    percents: impl Iterator<Item = Option<&'a str>>,
) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    let gutters = n.saturating_sub(1);
    let inner = total.saturating_sub(gutters).max(n);
    let parsed: Vec<Option<u32>> = percents.map(parse_pct).collect();
    if parsed.len() == n && parsed.iter().all(|p| p.is_some()) {
        let sum: u32 = parsed.iter().map(|p| p.unwrap()).sum();
        if sum > 0 {
            let mut widths: Vec<usize> = parsed
                .iter()
                .map(|p| ((inner as u32 * p.unwrap()) / sum) as usize)
                .collect();
            for w in &mut widths {
                if *w == 0 {
                    *w = 1;
                }
            }
            let used: usize = widths.iter().sum();
            if inner >= used {
                if let Some(last) = widths.last_mut() {
                    *last += inner - used;
                }
            }
            return widths;
        }
    }
    let each = inner / n;
    let rem = inner % n;
    (0..n)
        .map(|i| (each + if i + 1 == n { rem } else { 0 }).max(1))
        .collect()
}

fn parse_pct(s: Option<&str>) -> Option<u32> {
    s?.trim().strip_suffix('%')?.parse().ok()
}

#[derive(Clone)]
struct Cell {
    ch: char,
    id: Option<TreeId>,
}

struct Canvas {
    width: usize,
    rows: Vec<Vec<Cell>>,
}

impl Canvas {
    fn new(width: usize) -> Self {
        Self {
            width: width.max(1),
            rows: Vec::new(),
        }
    }

    fn height(&self) -> usize {
        self.rows.len()
    }

    fn ensure(&mut self, y: usize) {
        while self.rows.len() <= y {
            self.rows.push(vec![Cell { ch: ' ', id: None }; self.width]);
        }
    }

    fn set(&mut self, x: usize, y: usize, ch: char, id: Option<&TreeId>) {
        if x >= self.width {
            return;
        }
        self.ensure(y);
        self.rows[y][x] = Cell {
            ch,
            id: id.cloned(),
        };
    }

    fn fill_line(&mut self, x: usize, y: usize, w: usize, text: &str, id: Option<&TreeId>) {
        if w == 0 {
            return;
        }
        let max = w.min(self.width.saturating_sub(x));
        let t: String = text.chars().take(max.saturating_sub(1)).collect();
        let padded: String = format!(" {t:<width$}", width = max.saturating_sub(1))
            .chars()
            .take(max)
            .collect();
        for (i, ch) in padded.chars().enumerate() {
            self.set(x + i, y, ch, id);
        }
        for i in padded.chars().count()..max {
            self.set(x + i, y, ' ', id);
        }
    }

    fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, id: Option<&TreeId>) {
        if w < 2 || h < 2 {
            return;
        }
        for i in 0..w {
            let ch = if i == 0 || i + 1 == w { '+' } else { '-' };
            self.set(x + i, y, ch, id);
            self.set(x + i, y + h - 1, ch, id);
        }
        for j in 1..h.saturating_sub(1) {
            self.set(x, y + j, '|', id);
            self.set(x + w - 1, y + j, '|', id);
        }
    }

    fn blit(&mut self, src: &Canvas, x: usize, y: usize) {
        for (sy, row) in src.rows.iter().enumerate() {
            for (sx, cell) in row.iter().enumerate() {
                if cell.ch == ' ' && cell.id.is_none() {
                    continue;
                }
                self.set(x + sx, y + sy, cell.ch, cell.id.as_ref());
            }
        }
    }

    fn fill_missing(&mut self, id: &TreeId) {
        for row in &mut self.rows {
            for cell in row {
                if cell.id.is_none() {
                    cell.id = Some(id.clone());
                }
            }
        }
    }

    fn finish(self) -> (Vec<String>, Vec<DetailHit>) {
        let mut lines = Vec::new();
        let mut hits = Vec::new();
        for (y, row) in self.rows.iter().enumerate() {
            lines.push(row.iter().map(|c| c.ch).collect());
            let mut x = 0;
            while x < row.len() {
                let Some(id) = row[x].id.clone() else {
                    x += 1;
                    continue;
                };
                let x0 = x;
                x += 1;
                while x < row.len() && row[x].id.as_ref() == Some(&id) {
                    x += 1;
                }
                hits.push(DetailHit {
                    line: y,
                    x0,
                    x1: x,
                    id,
                });
            }
        }
        (lines, hits)
    }
}

fn dash(s: &str) -> &str {
    if s.trim().is_empty() { "—" } else { s }
}

fn truncate(s: &str, max: usize) -> String {
    let t = s.replace('\n', " ");
    if t.chars().count() <= max {
        t
    } else {
        let mut out: String = t.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn host_of(href: &str) -> &str {
    href.split('/').nth(2).unwrap_or(href)
}

fn json_ld_summary(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return "(empty)".into();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(t) = v.get("@type").and_then(|x| x.as_str()) {
            return t.to_string();
        }
        if let Some(arr) = v.as_array() {
            if let Some(t) = arr
                .iter()
                .find_map(|x| x.get("@type").and_then(|y| y.as_str()))
            {
                return t.to_string();
            }
        }
    }
    "(invalid JSON)".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MjText, Template};

    fn body_row() -> TreeRow {
        TreeRow {
            id: TreeId::Body,
            label: "[BODY] mj-body".into(),
            prefix: String::new(),
            expandable: true,
        }
    }

    fn sample() -> Template {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![SectionChild::MjColumn(MjColumn {
                width: Some("100%".into()),
                background_color: None,
                padding: None,
                inner_background_color: None,
                components: vec![
                    ColumnChild::MjText(MjText {
                        content: "Hello".into(),
                        align: None,
                        font_size: None,
                        font_family: None,
                        color: None,
                        padding: None,
                        ..Default::default()
                    }),
                    ColumnChild::MjButton(crate::model::MjButton {
                        content: "Go".into(),
                        href: "https://example.com".into(),
                        background_color: None,
                        color: None,
                        align: None,
                        font_family: None,
                        border_radius: None,
                        width: None,
                        padding: None,
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            })],
            ..Default::default()
        }));
        t
    }

    #[test]
    fn blueprint_lists_nested_layout() {
        let t = sample();
        let (lines, hits) = details_view(Some(&t), Some(&body_row()), 48);
        let joined = lines.join("\n");
        assert!(
            joined.contains("mj-section") || joined.contains("mj-column"),
            "{joined}"
        );
        assert!(joined.contains("mj-text"), "{joined}");
        assert!(joined.contains("mj-button"), "{joined}");
        assert!(hits.iter().any(|h| matches!(
            &h.id,
            TreeId::Path(p) if matches!(p.last(), Some(Step::ColComp(_)))
        )));
        assert!(hits.iter().any(|h| matches!(
            &h.id,
            TreeId::Path(p) if matches!(p.as_slice(), [Step::BodyNode(_)])
        )));
    }

    #[test]
    fn blueprint_shows_section_and_wrapper_labels() {
        let mut t = Template::minimal();
        t.body.nodes.push(BodyNode::MjSection(MjSection {
            label: Some("hero".into()),
            children: vec![SectionChild::MjColumn(MjColumn {
                components: vec![ColumnChild::MjText(MjText {
                    content: "Hi".into(),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }));
        t.body.nodes.push(BodyNode::MjWrapper(MjWrapper {
            label: Some("footer".into()),
            ..Default::default()
        }));
        let (lines, _) = details_view(Some(&t), Some(&body_row()), 48);
        let joined = lines.join("\n");
        assert!(joined.contains("hero"), "{joined}");
        assert!(joined.contains("mj-wrapper  footer"), "{joined}");
        let section_row = TreeRow {
            id: TreeId::Path(vec![Step::BodyNode(0)]),
            label: "1. mj-section  hero".into(),
            prefix: String::new(),
            expandable: true,
        };
        let summary = details_lines(Some(&t), Some(&section_row), 40);
        assert!(summary.iter().any(|l| l == "label: hero"), "{summary:?}");
    }

    #[test]
    fn blueprint_stays_full_when_a_leaf_is_selected() {
        let t = sample();
        let row = TreeRow {
            id: TreeId::Path(vec![
                Step::BodyNode(0),
                Step::SectionChild(0),
                Step::ColComp(1),
            ]),
            label: "mj-button".into(),
            prefix: String::new(),
            expandable: false,
        };
        let (lines, hits) = details_view(Some(&t), Some(&row), 48);
        let joined = lines.join("\n");
        assert!(joined.contains("mj-text"), "{joined}");
        assert!(joined.contains("mj-button"), "{joined}");
        assert!(hits.iter().any(|h| matches!(
            &h.id,
            TreeId::Path(p) if matches!(p.last(), Some(Step::ColComp(0)))
        )));
    }
}
