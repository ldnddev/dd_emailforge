//! Populate FormEdit from a tree selection and write it back on Ctrl+S.
use anyhow::{Context, Result, anyhow};

use crate::model::{
    Align, BodyNode, ColumnChild, EmailArticle, EmailCta, EmailFooter, EmailHeader, EmailHero,
    HeroMode, ImagePosition, MjAccordion, MjAccordionElement, MjCarousel, MjCarouselImage,
    MjColumn, MjGroup, MjHero, MjNavbar, MjNavbarLink, MjSocial, MjSocialElement, SectionChild,
    SocialMode, SocialNetwork, Template, Thumbnails, WebFont,
};

use super::editform::{self, EditFormState};
use super::tree::{Step, TreeId};

pub fn form_for(template: &Template, id: &TreeId) -> Option<EditFormState> {
    match id {
        TreeId::Head => Some(head_to_form(template)),
        TreeId::Brand => Some(brand_to_form(template)),
        TreeId::Body => Some(body_to_form(template)),
        TreeId::Path(path) => path_to_form(template, path),
    }
}

pub fn apply_form(template: &mut Template, id: &TreeId, state: &EditFormState) -> Result<()> {
    match id {
        TreeId::Head => apply_head(template, state),
        TreeId::Brand => apply_brand(template, state),
        TreeId::Body => apply_body(template, state),
        TreeId::Path(path) => apply_path(template, path, state),
    }
}

fn head_to_form(t: &Template) -> EditFormState {
    let mut s = EditFormState::new(&editform::HEAD_FORM);
    s.set("subject", t.subject.clone());
    s.set("preheader", t.preheader.clone());
    s.set("lang", t.lang.clone());
    s.set("dir", t.dir.clone());
    s.set("title", t.head.title.clone());
    s.set("breakpoint", t.head.breakpoint.clone());
    s.set("base_url", t.base_url.clone());
    s.set("json_ld", t.head.json_ld.clone());
    s.set("css", t.head.css.clone());
    s.set("css_inline", bool_str(t.head.css_inline));
    let mut fonts = Vec::new();
    for font in &t.head.fonts {
        let mut item = EditFormState::new(&editform::FONT_ITEM_FORM);
        item.set("name", font.name.clone());
        item.set("href", font.href.clone());
        fonts.push(item);
    }
    s.sub_state.insert("fonts".into(), fonts);
    s.selected_sub_item.insert("fonts".into(), 0);
    s
}

fn apply_head(t: &mut Template, state: &EditFormState) -> Result<()> {
    t.subject = state.get("subject").to_string();
    t.preheader = state.get("preheader").to_string();
    t.lang = state.get("lang").to_string();
    t.dir = state.get("dir").trim().to_string();
    t.head.title = state.get("title").to_string();
    t.head.breakpoint = state.get("breakpoint").to_string();
    t.base_url = state.get("base_url").to_string();
    t.head.json_ld = state.get("json_ld").to_string();
    t.head.css = state.get("css").to_string();
    t.head.css_inline = parse_bool(state.get("css_inline"))?;
    t.head.fonts.clear();
    if let Some(items) = state.sub_state.get("fonts") {
        for item in items {
            t.head.fonts.push(WebFont {
                name: item.get("name").trim().to_string(),
                href: item.get("href").trim().to_string(),
            });
        }
    }
    Ok(())
}

fn brand_to_form(t: &Template) -> EditFormState {
    let mut s = EditFormState::new(&editform::BRAND_FORM);
    s.set("font_family", t.brand.font_family.clone());
    s.set("text_color", t.brand.text_color.clone());
    s.set("background_color", t.brand.background_color.clone());
    s.set("content_width", t.brand.content_width.to_string());
    s.set("button_background", t.brand.button_background.clone());
    s.set("button_color", t.brand.button_color.clone());
    s
}

fn apply_brand(t: &mut Template, state: &EditFormState) -> Result<()> {
    t.brand.font_family = state.get("font_family").to_string();
    t.brand.text_color = state.get("text_color").trim().to_string();
    t.brand.background_color = state.get("background_color").trim().to_string();
    t.brand.content_width = state
        .get("content_width")
        .trim()
        .parse::<u32>()
        .context("content_width must be a number")?;
    t.brand.button_background = state.get("button_background").trim().to_string();
    t.brand.button_color = state.get("button_color").trim().to_string();
    Ok(())
}

fn body_to_form(t: &Template) -> EditFormState {
    let mut s = EditFormState::new(&editform::BODY_FORM);
    s.set("background_color", t.body.background_color.clone());
    s.set("css_class", opt_get(&t.body.css_class));
    s
}

fn apply_body(t: &mut Template, state: &EditFormState) -> Result<()> {
    t.body.background_color = state.get("background_color").trim().to_string();
    t.body.css_class = opt_set(state, "css_class");
    Ok(())
}

fn path_to_form(t: &Template, path: &[Step]) -> Option<EditFormState> {
    match locate(t, path)? {
        Located::BodyNode(n) => Some(body_node_to_form(n)),
        Located::Column(c) => Some(column_to_form(c)),
        Located::Group(g) => Some(group_to_form(g)),
        Located::Leaf(c) => Some(leaf_to_form(c)),
        Located::NavbarLink(l) => Some(navbar_link_to_form(l)),
        Located::AccordionEl(e) => Some(accordion_el_to_form(e)),
        Located::CarouselImg(i) => Some(carousel_img_to_form(i)),
    }
}

fn apply_path(t: &mut Template, path: &[Step], state: &EditFormState) -> Result<()> {
    match locate_mut(t, path).ok_or_else(|| anyhow!("missing node"))? {
        LocatedMut::BodyNode(n) => apply_body_node(n, state),
        LocatedMut::Column(c) => apply_column(c, state),
        LocatedMut::Group(g) => apply_group(g, state),
        LocatedMut::Leaf(c) => apply_leaf(c, state),
        LocatedMut::NavbarLink(l) => apply_navbar_link(l, state),
        LocatedMut::AccordionEl(e) => apply_accordion_el(e, state),
        LocatedMut::CarouselImg(i) => apply_carousel_img(i, state),
    }
}

enum Located<'a> {
    BodyNode(&'a BodyNode),
    Column(&'a MjColumn),
    Group(&'a MjGroup),
    Leaf(&'a ColumnChild),
    NavbarLink(&'a MjNavbarLink),
    AccordionEl(&'a MjAccordionElement),
    CarouselImg(&'a MjCarouselImage),
}

enum LocatedMut<'a> {
    BodyNode(&'a mut BodyNode),
    Column(&'a mut MjColumn),
    Group(&'a mut MjGroup),
    Leaf(&'a mut ColumnChild),
    NavbarLink(&'a mut MjNavbarLink),
    AccordionEl(&'a mut MjAccordionElement),
    CarouselImg(&'a mut MjCarouselImage),
}

fn locate<'a>(t: &'a Template, path: &[Step]) -> Option<Located<'a>> {
    let mut steps = path.iter();
    let Step::BodyNode(i) = steps.next()? else {
        return None;
    };
    let mut node = t.body.nodes.get(*i)?;
    loop {
        match steps.next() {
            None => return Some(Located::BodyNode(node)),
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
                    SectionChild::MjColumn(c) => locate_column(c, steps),
                    SectionChild::MjGroup(g) => locate_group(g, steps),
                };
            }
            Some(Step::HeroChild(j)) => {
                let BodyNode::MjHero(h) = node else {
                    return None;
                };
                return locate_column_child(h.children.get(*j)?, steps);
            }
            _ => return None,
        }
    }
}

fn locate_column<'a>(
    c: &'a MjColumn,
    mut steps: std::slice::Iter<'_, Step>,
) -> Option<Located<'a>> {
    match steps.next() {
        None => Some(Located::Column(c)),
        Some(Step::ColComp(i)) => locate_column_child(c.components.get(*i)?, steps),
        _ => None,
    }
}

fn locate_column_child<'a>(
    c: &'a ColumnChild,
    mut steps: std::slice::Iter<'_, Step>,
) -> Option<Located<'a>> {
    match steps.next() {
        None => Some(Located::Leaf(c)),
        Some(Step::NavbarLink(i)) => {
            let ColumnChild::MjNavbar(n) = c else {
                return None;
            };
            Some(Located::NavbarLink(n.links.get(*i)?))
        }
        Some(Step::AccordionEl(i)) => {
            let ColumnChild::MjAccordion(n) = c else {
                return None;
            };
            Some(Located::AccordionEl(n.elements.get(*i)?))
        }
        Some(Step::CarouselImg(i)) => {
            let ColumnChild::MjCarousel(n) = c else {
                return None;
            };
            Some(Located::CarouselImg(n.images.get(*i)?))
        }
        _ => None,
    }
}

fn locate_group<'a>(g: &'a MjGroup, mut steps: std::slice::Iter<'_, Step>) -> Option<Located<'a>> {
    match steps.next() {
        None => Some(Located::Group(g)),
        Some(Step::GroupCol(i)) => locate_column(g.children.get(*i)?, steps),
        _ => None,
    }
}

fn locate_mut<'a>(t: &'a mut Template, path: &[Step]) -> Option<LocatedMut<'a>> {
    let mut steps = path.iter();
    let Step::BodyNode(i) = steps.next()? else {
        return None;
    };
    let mut node = t.body.nodes.get_mut(*i)?;
    loop {
        match steps.next() {
            None => return Some(LocatedMut::BodyNode(node)),
            Some(Step::WrapperChild(j)) => {
                let BodyNode::MjWrapper(w) = node else {
                    return None;
                };
                node = w.children.get_mut(*j)?;
            }
            Some(Step::SectionChild(j)) => {
                let BodyNode::MjSection(s) = node else {
                    return None;
                };
                return match s.children.get_mut(*j)? {
                    SectionChild::MjColumn(c) => locate_column_mut(c, steps),
                    SectionChild::MjGroup(g) => locate_group_mut(g, steps),
                };
            }
            Some(Step::HeroChild(j)) => {
                let BodyNode::MjHero(h) = node else {
                    return None;
                };
                return locate_column_child_mut(h.children.get_mut(*j)?, steps);
            }
            _ => return None,
        }
    }
}

fn locate_column_mut<'a>(
    c: &'a mut MjColumn,
    mut steps: std::slice::Iter<'_, Step>,
) -> Option<LocatedMut<'a>> {
    match steps.next() {
        None => Some(LocatedMut::Column(c)),
        Some(Step::ColComp(i)) => locate_column_child_mut(c.components.get_mut(*i)?, steps),
        _ => None,
    }
}

fn locate_column_child_mut<'a>(
    c: &'a mut ColumnChild,
    mut steps: std::slice::Iter<'_, Step>,
) -> Option<LocatedMut<'a>> {
    match steps.next() {
        None => Some(LocatedMut::Leaf(c)),
        Some(Step::NavbarLink(i)) => {
            let ColumnChild::MjNavbar(n) = c else {
                return None;
            };
            Some(LocatedMut::NavbarLink(n.links.get_mut(*i)?))
        }
        Some(Step::AccordionEl(i)) => {
            let ColumnChild::MjAccordion(n) = c else {
                return None;
            };
            Some(LocatedMut::AccordionEl(n.elements.get_mut(*i)?))
        }
        Some(Step::CarouselImg(i)) => {
            let ColumnChild::MjCarousel(n) = c else {
                return None;
            };
            Some(LocatedMut::CarouselImg(n.images.get_mut(*i)?))
        }
        _ => None,
    }
}

fn locate_group_mut<'a>(
    g: &'a mut MjGroup,
    mut steps: std::slice::Iter<'_, Step>,
) -> Option<LocatedMut<'a>> {
    match steps.next() {
        None => Some(LocatedMut::Group(g)),
        Some(Step::GroupCol(i)) => locate_column_mut(g.children.get_mut(*i)?, steps),
        _ => None,
    }
}

fn body_node_to_form(n: &BodyNode) -> EditFormState {
    match n {
        BodyNode::MjSection(s) => {
            let mut st = EditFormState::new(&editform::SECTION_FORM);
            st.set("label", opt_get(&s.label));
            st.set("background_color", opt_get(&s.background_color));
            st.set("background_url", opt_get(&s.background_url));
            st.set("background_size", opt_get(&s.background_size));
            st.set("background_repeat", opt_get(&s.background_repeat));
            st.set("padding", opt_get(&s.padding));
            st.set("gutter", opt_get(&s.gutter));
            st.set("direction", opt_get(&s.direction));
            st.set("border", opt_get(&s.border));
            st.set("border_sides", sides_to_form(&s.border_sides));
            st.set("border_radius", opt_get(&s.border_radius));
            st.set("full_width", bool_str(s.full_width));
            st.set("css_class", opt_get(&s.css_class));
            st
        }
        BodyNode::MjWrapper(w) => {
            let mut st = EditFormState::new(&editform::WRAPPER_FORM);
            st.set("label", opt_get(&w.label));
            st.set("background_color", opt_get(&w.background_color));
            st.set("background_url", opt_get(&w.background_url));
            st.set("background_size", opt_get(&w.background_size));
            st.set("background_repeat", opt_get(&w.background_repeat));
            st.set("padding", opt_get(&w.padding));
            st.set("gap", opt_get(&w.gap));
            st.set("border", opt_get(&w.border));
            st.set("border_sides", sides_to_form(&w.border_sides));
            st.set("border_radius", opt_get(&w.border_radius));
            st.set("full_width", bool_str(w.full_width));
            st.set("css_class", opt_get(&w.css_class));
            st
        }
        BodyNode::MjHero(h) => hero_to_form(h),
        BodyNode::EmailHeader(h) => email_header_to_form(h),
        BodyNode::EmailHero(h) => email_hero_to_form(h),
        BodyNode::EmailCta(c) => email_cta_to_form(c),
        BodyNode::EmailArticle(a) => email_article_to_form(a),
        BodyNode::EmailFooter(f) => email_footer_to_form(f),
    }
}

fn apply_body_node(n: &mut BodyNode, state: &EditFormState) -> Result<()> {
    match n {
        BodyNode::MjSection(s) => {
            s.label = opt_set(state, "label");
            s.background_color = opt_set(state, "background_color");
            s.background_url = opt_set(state, "background_url");
            s.background_size = opt_set(state, "background_size");
            s.background_repeat = opt_set(state, "background_repeat");
            s.padding = opt_set_padding(state)?;
            s.gutter = opt_set_unit(state, "gutter")?;
            s.direction = opt_set(state, "direction");
            s.border = opt_set(state, "border");
            s.border_sides = opt_set_sides(state, "border_sides")?;
            s.border_radius = opt_set_unit(state, "border_radius")?;
            s.full_width = parse_bool(state.get("full_width"))?;
            s.css_class = opt_set(state, "css_class");
            Ok(())
        }
        BodyNode::MjWrapper(w) => {
            w.label = opt_set(state, "label");
            w.background_color = opt_set(state, "background_color");
            w.background_url = opt_set(state, "background_url");
            w.background_size = opt_set(state, "background_size");
            w.background_repeat = opt_set(state, "background_repeat");
            w.padding = opt_set_padding(state)?;
            w.gap = opt_set_unit(state, "gap")?;
            w.border = opt_set(state, "border");
            w.border_sides = opt_set_sides(state, "border_sides")?;
            w.border_radius = opt_set_unit(state, "border_radius")?;
            w.full_width = parse_bool(state.get("full_width"))?;
            w.css_class = opt_set(state, "css_class");
            Ok(())
        }
        BodyNode::MjHero(h) => apply_hero(h, state),
        BodyNode::EmailHeader(h) => apply_email_header(h, state),
        BodyNode::EmailHero(h) => apply_email_hero(h, state),
        BodyNode::EmailCta(c) => apply_email_cta(c, state),
        BodyNode::EmailArticle(a) => apply_email_article(a, state),
        BodyNode::EmailFooter(f) => apply_email_footer(f, state),
    }
}

fn column_to_form(c: &MjColumn) -> EditFormState {
    let mut st = EditFormState::new(&editform::COLUMN_FORM);
    st.set("width", opt_get(&c.width));
    st.set("background_color", opt_get(&c.background_color));
    st.set("padding", opt_get(&c.padding));
    st.set("inner_background_color", opt_get(&c.inner_background_color));
    st.set("border", opt_get(&c.border));
    st.set("border_sides", sides_to_form(&c.border_sides));
    st.set("border_radius", opt_get(&c.border_radius));
    st.set("inner_border", opt_get(&c.inner_border));
    st.set("inner_border_sides", sides_to_form(&c.inner_border_sides));
    st.set("inner_border_radius", opt_get(&c.inner_border_radius));
    st.set("vertical_align", opt_get(&c.vertical_align));
    st.set("css_class", opt_get(&c.css_class));
    st
}

fn apply_column(c: &mut MjColumn, state: &EditFormState) -> Result<()> {
    c.width = opt_set(state, "width");
    c.background_color = opt_set(state, "background_color");
    c.padding = opt_set_padding(state)?;
    c.inner_background_color = opt_set(state, "inner_background_color");
    c.border = opt_set(state, "border");
    c.border_sides = opt_set_sides(state, "border_sides")?;
    c.border_radius = opt_set_unit(state, "border_radius")?;
    c.inner_border = opt_set(state, "inner_border");
    c.inner_border_sides = opt_set_sides(state, "inner_border_sides")?;
    c.inner_border_radius = opt_set_unit(state, "inner_border_radius")?;
    c.vertical_align = opt_set(state, "vertical_align");
    c.css_class = opt_set(state, "css_class");
    Ok(())
}

fn group_to_form(g: &MjGroup) -> EditFormState {
    let mut st = EditFormState::new(&editform::GROUP_FORM);
    st.set("width", opt_get(&g.width));
    st.set("background_color", opt_get(&g.background_color));
    st.set("direction", opt_get(&g.direction));
    st.set("vertical_align", opt_get(&g.vertical_align));
    st.set("css_class", opt_get(&g.css_class));
    st
}

fn apply_group(g: &mut MjGroup, state: &EditFormState) -> Result<()> {
    g.width = opt_set(state, "width");
    g.background_color = opt_set(state, "background_color");
    g.direction = opt_set(state, "direction");
    g.vertical_align = opt_set(state, "vertical_align");
    g.css_class = opt_set(state, "css_class");
    Ok(())
}

fn hero_to_form(h: &MjHero) -> EditFormState {
    let mut st = EditFormState::new(&editform::HERO_FORM);
    st.set("mode", hero_mode_str(h.mode));
    st.set("background_url", opt_get(&h.background_url));
    st.set("background_color", opt_get(&h.background_color));
    st.set("background_height", opt_get(&h.background_height));
    st.set("background_width", opt_get(&h.background_width));
    st.set("background_position", opt_get(&h.background_position));
    st.set("width", opt_get(&h.width));
    st.set("height", opt_get(&h.height));
    st.set("padding", opt_get(&h.padding));
    st.set("inner_padding", opt_get(&h.inner_padding));
    st.set("border_radius", opt_get(&h.border_radius));
    st.set("vertical_align", opt_get(&h.vertical_align));
    st.set("css_class", opt_get(&h.css_class));
    st
}

fn apply_hero(h: &mut MjHero, state: &EditFormState) -> Result<()> {
    h.mode = parse_hero_mode(state.get("mode"))?;
    h.background_url = opt_set(state, "background_url");
    h.background_color = opt_set(state, "background_color");
    h.background_height = opt_set(state, "background_height");
    h.background_width = opt_set(state, "background_width");
    h.background_position = opt_set(state, "background_position");
    h.width = opt_set(state, "width");
    h.height = opt_set(state, "height");
    h.padding = opt_set_padding(state)?;
    h.inner_padding = opt_set_named_padding(state, "inner_padding")?;
    h.border_radius = opt_set_unit(state, "border_radius")?;
    h.vertical_align = opt_set(state, "vertical_align");
    h.css_class = opt_set(state, "css_class");
    Ok(())
}

fn leaf_to_form(c: &ColumnChild) -> EditFormState {
    match c {
        ColumnChild::MjText(t) => {
            let mut st = EditFormState::new(&editform::TEXT_FORM);
            st.set("content", t.content.clone());
            st.set("align", align_str(t.align));
            st.set("font_size", opt_get(&t.font_size));
            st.set("font_family", opt_get(&t.font_family));
            st.set("font_weight", opt_get(&t.font_weight));
            st.set("font_style", opt_get(&t.font_style));
            st.set("line_height", opt_get(&t.line_height));
            st.set("color", opt_get(&t.color));
            st.set("padding", opt_get(&t.padding));
            st.set("letter_spacing", opt_get(&t.letter_spacing));
            st.set("text_decoration", opt_get(&t.text_decoration));
            st.set("text_transform", opt_get(&t.text_transform));
            st.set("height", opt_get(&t.height));
            st.set("css_class", opt_get(&t.css_class));
            st
        }
        ColumnChild::MjButton(b) => {
            let mut st = EditFormState::new(&editform::BUTTON_FORM);
            st.set("content", b.content.clone());
            st.set("href", b.href.clone());
            st.set("background_color", opt_get(&b.background_color));
            st.set("color", opt_get(&b.color));
            st.set("align", align_str(b.align));
            st.set("font_family", opt_get(&b.font_family));
            st.set("font_size", opt_get(&b.font_size));
            st.set("font_weight", opt_get(&b.font_weight));
            st.set("font_style", opt_get(&b.font_style));
            st.set("border", opt_get(&b.border));
            st.set("border_sides", sides_to_form(&b.border_sides));
            st.set("border_radius", opt_get(&b.border_radius));
            st.set("inner_padding", opt_get(&b.inner_padding));
            st.set("width", opt_get(&b.width));
            st.set("height", opt_get(&b.height));
            st.set("target", opt_get(&b.target));
            st.set("padding", opt_get(&b.padding));
            st.set("letter_spacing", opt_get(&b.letter_spacing));
            st.set("line_height", opt_get(&b.line_height));
            st.set("text_decoration", opt_get(&b.text_decoration));
            st.set("text_transform", opt_get(&b.text_transform));
            st.set("rel", opt_get(&b.rel));
            st.set("title", opt_get(&b.title));
            st.set("css_class", opt_get(&b.css_class));
            st
        }
        ColumnChild::MjImage(i) => {
            let mut st = EditFormState::new(&editform::IMAGE_FORM);
            st.set("src", i.src.clone());
            st.set("alt", i.alt.clone());
            st.set("href", opt_get(&i.href));
            st.set("title", opt_get(&i.title));
            st.set("width", opt_get(&i.width));
            st.set("height", opt_get(&i.height));
            st.set("align", align_str(i.align));
            st.set("fluid_on_mobile", bool_str(i.fluid_on_mobile));
            st.set("border", opt_get(&i.border));
            st.set("border_sides", sides_to_form(&i.border_sides));
            st.set("border_radius", opt_get(&i.border_radius));
            st.set("padding", opt_get(&i.padding));
            st.set("target", opt_get(&i.target));
            st.set("rel", opt_get(&i.rel));
            st.set("css_class", opt_get(&i.css_class));
            st
        }
        ColumnChild::MjDivider(d) => {
            let mut st = EditFormState::new(&editform::DIVIDER_FORM);
            st.set("border_color", opt_get(&d.border_color));
            st.set("border_width", opt_get(&d.border_width));
            st.set("border_style", opt_get(&d.border_style));
            st.set("width", opt_get(&d.width));
            st.set("align", align_str(d.align));
            st.set("padding", opt_get(&d.padding));
            st.set("css_class", opt_get(&d.css_class));
            st
        }
        ColumnChild::MjSpacer(s) => {
            let mut st = EditFormState::new(&editform::SPACER_FORM);
            st.set("height", s.height.clone());
            st.set("padding", opt_get(&s.padding));
            st.set("css_class", opt_get(&s.css_class));
            st
        }
        ColumnChild::MjSocial(s) => social_to_form(s),
        ColumnChild::MjTable(tb) => {
            let mut st = EditFormState::new(&editform::TABLE_FORM);
            st.set("content", tb.content.clone());
            st.set("font_size", opt_get(&tb.font_size));
            st.set("font_family", opt_get(&tb.font_family));
            st.set("line_height", opt_get(&tb.line_height));
            st.set("color", opt_get(&tb.color));
            st.set("align", align_str(tb.align));
            st.set("width", opt_get(&tb.width));
            st.set("border", opt_get(&tb.border));
            st.set("padding", opt_get(&tb.padding));
            st.set("cellpadding", opt_get(&tb.cellpadding));
            st.set("cellspacing", opt_get(&tb.cellspacing));
            st.set("role", opt_get(&tb.role));
            st.set("css_class", opt_get(&tb.css_class));
            st
        }
        ColumnChild::MjNavbar(n) => navbar_to_form(n),
        ColumnChild::MjAccordion(a) => accordion_to_form(a),
        ColumnChild::MjCarousel(c) => carousel_to_form(c),
    }
}

fn apply_leaf(c: &mut ColumnChild, state: &EditFormState) -> Result<()> {
    match c {
        ColumnChild::MjText(t) => {
            t.content = state.get("content").to_string();
            t.align = parse_align(state.get("align"))?;
            t.font_size = opt_set(state, "font_size");
            t.font_family = opt_set(state, "font_family");
            t.font_weight = opt_set(state, "font_weight");
            t.font_style = opt_set(state, "font_style");
            t.line_height = opt_set(state, "line_height");
            t.color = opt_set(state, "color");
            t.padding = opt_set_padding(state)?;
            t.letter_spacing = opt_set(state, "letter_spacing");
            t.text_decoration = opt_set(state, "text_decoration");
            t.text_transform = opt_set(state, "text_transform");
            t.height = opt_set(state, "height");
            t.css_class = opt_set(state, "css_class");
            Ok(())
        }
        ColumnChild::MjButton(b) => {
            b.content = state.get("content").to_string();
            b.href = state.get("href").to_string();
            b.background_color = opt_set(state, "background_color");
            b.color = opt_set(state, "color");
            b.align = parse_align(state.get("align"))?;
            b.font_family = opt_set(state, "font_family");
            b.font_size = opt_set(state, "font_size");
            b.font_weight = opt_set(state, "font_weight");
            b.font_style = opt_set(state, "font_style");
            b.border = opt_set(state, "border");
            b.border_sides = opt_set_sides(state, "border_sides")?;
            b.border_radius = opt_set_unit(state, "border_radius")?;
            b.inner_padding = opt_set_named_padding(state, "inner_padding")?;
            b.width = opt_set(state, "width");
            b.height = opt_set(state, "height");
            b.target = opt_set(state, "target");
            b.padding = opt_set_padding(state)?;
            b.letter_spacing = opt_set(state, "letter_spacing");
            b.line_height = opt_set(state, "line_height");
            b.text_decoration = opt_set(state, "text_decoration");
            b.text_transform = opt_set(state, "text_transform");
            b.rel = opt_set(state, "rel");
            b.title = opt_set(state, "title");
            b.css_class = opt_set(state, "css_class");
            Ok(())
        }
        ColumnChild::MjImage(i) => {
            i.src = state.get("src").trim().to_string();
            i.alt = state.get("alt").to_string();
            i.href = opt_set(state, "href");
            i.title = opt_set(state, "title");
            i.width = opt_set(state, "width");
            i.height = opt_set(state, "height");
            i.align = parse_align(state.get("align"))?;
            i.fluid_on_mobile = parse_bool(state.get("fluid_on_mobile"))?;
            i.border = opt_set(state, "border");
            i.border_sides = opt_set_sides(state, "border_sides")?;
            i.border_radius = opt_set_unit(state, "border_radius")?;
            i.padding = opt_set_padding(state)?;
            i.target = opt_set(state, "target");
            i.rel = opt_set(state, "rel");
            i.css_class = opt_set(state, "css_class");
            Ok(())
        }
        ColumnChild::MjDivider(d) => {
            d.border_color = opt_set(state, "border_color");
            d.border_width = opt_set(state, "border_width");
            d.border_style = opt_set(state, "border_style");
            d.width = opt_set(state, "width");
            d.align = parse_align(state.get("align"))?;
            d.padding = opt_set_padding(state)?;
            d.css_class = opt_set(state, "css_class");
            Ok(())
        }
        ColumnChild::MjSpacer(s) => {
            s.height = state.get("height").trim().to_string();
            s.padding = opt_set_padding(state)?;
            s.css_class = opt_set(state, "css_class");
            Ok(())
        }
        ColumnChild::MjSocial(s) => apply_social(s, state),
        ColumnChild::MjTable(tb) => {
            tb.content = state.get("content").to_string();
            tb.font_size = opt_set(state, "font_size");
            tb.font_family = opt_set(state, "font_family");
            tb.line_height = opt_set(state, "line_height");
            tb.color = opt_set(state, "color");
            tb.align = parse_align(state.get("align"))?;
            tb.width = opt_set(state, "width");
            tb.border = opt_set(state, "border");
            tb.padding = opt_set_padding(state)?;
            tb.cellpadding = opt_set(state, "cellpadding");
            tb.cellspacing = opt_set(state, "cellspacing");
            tb.role = opt_set(state, "role");
            tb.css_class = opt_set(state, "css_class");
            Ok(())
        }
        ColumnChild::MjNavbar(n) => apply_navbar(n, state),
        ColumnChild::MjAccordion(a) => apply_accordion(a, state),
        ColumnChild::MjCarousel(c) => apply_carousel(c, state),
    }
}

fn navbar_to_form(n: &MjNavbar) -> EditFormState {
    let mut st = EditFormState::new(&editform::NAVBAR_FORM);
    st.set("hamburger", bool_str(n.hamburger));
    st.set("ico_color", opt_get(&n.ico_color));
    st.set("base_url", opt_get(&n.base_url));
    st.set("align", align_str(n.align));
    st.set("padding", opt_get(&n.padding));
    st.set("ico_align", opt_get(&n.ico_align));
    st.set("ico_font_size", opt_get(&n.ico_font_size));
    st.set("ico_padding", opt_get(&n.ico_padding));
    st.set("ico_open", opt_get(&n.ico_open));
    st.set("ico_close", opt_get(&n.ico_close));
    st.set("css_class", opt_get(&n.css_class));
    st
}

fn apply_navbar(n: &mut MjNavbar, state: &EditFormState) -> Result<()> {
    n.hamburger = parse_bool(state.get("hamburger"))?;
    n.ico_color = opt_set(state, "ico_color");
    n.base_url = opt_set(state, "base_url");
    n.align = parse_align(state.get("align"))?;
    n.padding = opt_set_padding(state)?;
    n.ico_align = opt_set(state, "ico_align");
    n.ico_font_size = opt_set(state, "ico_font_size");
    n.ico_padding = opt_set_named_padding(state, "ico_padding")?;
    n.ico_open = opt_set(state, "ico_open");
    n.ico_close = opt_set(state, "ico_close");
    n.css_class = opt_set(state, "css_class");
    Ok(())
}

fn navbar_link_to_form(l: &MjNavbarLink) -> EditFormState {
    let mut st = EditFormState::new(&editform::NAVBAR_LINK_FORM);
    st.set("href", l.href.clone());
    st.set("content", l.content.clone());
    st.set("color", opt_get(&l.color));
    st.set("font_family", opt_get(&l.font_family));
    st.set("font_size", opt_get(&l.font_size));
    st.set("font_weight", opt_get(&l.font_weight));
    st.set("text_decoration", opt_get(&l.text_decoration));
    st.set("text_transform", opt_get(&l.text_transform));
    st.set("padding", opt_get(&l.padding));
    st.set("css_class", opt_get(&l.css_class));
    st
}

fn apply_navbar_link(l: &mut MjNavbarLink, state: &EditFormState) -> Result<()> {
    l.href = state.get("href").to_string();
    l.content = state.get("content").to_string();
    l.color = opt_set(state, "color");
    l.font_family = opt_set(state, "font_family");
    l.font_size = opt_set(state, "font_size");
    l.font_weight = opt_set(state, "font_weight");
    l.text_decoration = opt_set(state, "text_decoration");
    l.text_transform = opt_set(state, "text_transform");
    l.padding = opt_set_padding(state)?;
    l.css_class = opt_set(state, "css_class");
    Ok(())
}

fn accordion_to_form(a: &MjAccordion) -> EditFormState {
    let mut st = EditFormState::new(&editform::ACCORDION_FORM);
    st.set("border", opt_get(&a.border));
    st.set("padding", opt_get(&a.padding));
    st.set("font_family", opt_get(&a.font_family));
    st.set("icon_position", opt_get(&a.icon_position));
    st.set("icon_width", opt_get(&a.icon_width));
    st.set("icon_height", opt_get(&a.icon_height));
    st.set("icon_wrapped_url", opt_get(&a.icon_wrapped_url));
    st.set("icon_unwrapped_url", opt_get(&a.icon_unwrapped_url));
    st.set("css_class", opt_get(&a.css_class));
    st
}

fn apply_accordion(a: &mut MjAccordion, state: &EditFormState) -> Result<()> {
    a.border = opt_set(state, "border");
    a.padding = opt_set_padding(state)?;
    a.font_family = opt_set(state, "font_family");
    a.icon_position = opt_set(state, "icon_position");
    a.icon_width = opt_set(state, "icon_width");
    a.icon_height = opt_set(state, "icon_height");
    a.icon_wrapped_url = opt_set(state, "icon_wrapped_url");
    a.icon_unwrapped_url = opt_set(state, "icon_unwrapped_url");
    a.css_class = opt_set(state, "css_class");
    Ok(())
}

fn accordion_el_to_form(e: &MjAccordionElement) -> EditFormState {
    let mut st = EditFormState::new(&editform::ACCORDION_ELEMENT_FORM);
    st.set("title", e.title.clone());
    st.set("content", e.content.clone());
    st.set("background_color", opt_get(&e.background_color));
    st.set("css_class", opt_get(&e.css_class));
    st
}

fn apply_accordion_el(e: &mut MjAccordionElement, state: &EditFormState) -> Result<()> {
    e.title = state.get("title").to_string();
    e.content = state.get("content").to_string();
    e.background_color = opt_set(state, "background_color");
    e.css_class = opt_set(state, "css_class");
    Ok(())
}

fn carousel_to_form(c: &MjCarousel) -> EditFormState {
    let mut st = EditFormState::new(&editform::CAROUSEL_FORM);
    st.set("align", align_str(c.align));
    st.set("padding", opt_get(&c.padding));
    st.set("border_radius", opt_get(&c.border_radius));
    st.set("tb_border_radius", opt_get(&c.tb_border_radius));
    st.set("icon_width", opt_get(&c.icon_width));
    st.set("css_class", opt_get(&c.css_class));
    st.set("thumbnails", thumbnails_str(c.thumbnails));
    st
}

fn apply_carousel(c: &mut MjCarousel, state: &EditFormState) -> Result<()> {
    c.align = parse_align(state.get("align"))?;
    c.padding = opt_set_padding(state)?;
    c.border_radius = opt_set_unit(state, "border_radius")?;
    c.tb_border_radius = opt_set_unit(state, "tb_border_radius")?;
    c.icon_width = opt_set(state, "icon_width");
    c.css_class = opt_set(state, "css_class");
    c.thumbnails = parse_thumbnails(state.get("thumbnails"))?;
    Ok(())
}

fn carousel_img_to_form(i: &MjCarouselImage) -> EditFormState {
    let mut st = EditFormState::new(&editform::CAROUSEL_IMAGE_FORM);
    st.set("src", i.src.clone());
    st.set("alt", i.alt.clone());
    st.set("href", opt_get(&i.href));
    st.set("thumbnails_src", opt_get(&i.thumbnails_src));
    st.set("border_radius", opt_get(&i.border_radius));
    st.set("css_class", opt_get(&i.css_class));
    st
}

fn apply_carousel_img(i: &mut MjCarouselImage, state: &EditFormState) -> Result<()> {
    i.src = state.get("src").trim().to_string();
    i.alt = state.get("alt").to_string();
    i.href = opt_set(state, "href");
    i.thumbnails_src = opt_set(state, "thumbnails_src");
    i.border_radius = opt_set_unit(state, "border_radius")?;
    i.css_class = opt_set(state, "css_class");
    Ok(())
}

fn social_to_form(s: &MjSocial) -> EditFormState {
    let mut st = EditFormState::new(&editform::SOCIAL_FORM);
    st.set("mode", social_mode_str(s.mode));
    st.set("align", align_str(s.align));
    st.set("icon_size", s.icon_size.clone());
    st.set("border_radius", opt_get(&s.border_radius));
    st.set("padding", opt_get(&s.padding));
    st.set("icon_padding", opt_get(&s.icon_padding));
    st.set("inner_padding", opt_get(&s.inner_padding));
    st.set("font_size", opt_get(&s.font_size));
    st.set("color", opt_get(&s.color));
    st.set("css_class", opt_get(&s.css_class));
    st.sub_state
        .insert("elements".into(), social_items(&s.elements));
    st.selected_sub_item.insert("elements".into(), 0);
    st
}

fn apply_social(s: &mut MjSocial, state: &EditFormState) -> Result<()> {
    s.mode = parse_social_mode(state.get("mode"))?;
    s.align = parse_align(state.get("align"))?;
    s.icon_size = state.get("icon_size").trim().to_string();
    s.border_radius = opt_set_unit(state, "border_radius")?;
    s.padding = opt_set_padding(state)?;
    s.icon_padding = opt_set_named_padding(state, "icon_padding")?;
    s.inner_padding = opt_set_named_padding(state, "inner_padding")?;
    s.font_size = opt_set(state, "font_size");
    s.color = opt_set(state, "color");
    s.css_class = opt_set(state, "css_class");
    s.elements = parse_social_items(state.sub_state.get("elements"))?;
    Ok(())
}

fn email_header_to_form(h: &EmailHeader) -> EditFormState {
    let mut st = EditFormState::new(&editform::EMAIL_HEADER_FORM);
    st.set("logo_src", h.logo_src.clone());
    st.set("logo_alt", h.logo_alt.clone());
    st.set("logo_href", opt_get(&h.logo_href));
    st.set("logo_width", h.logo_width.clone());
    st.set("background_color", opt_get(&h.background_color));
    st
}

fn apply_email_header(h: &mut EmailHeader, state: &EditFormState) -> Result<()> {
    h.logo_src = state.get("logo_src").trim().to_string();
    h.logo_alt = state.get("logo_alt").to_string();
    h.logo_href = opt_set(state, "logo_href");
    h.logo_width = state.get("logo_width").trim().to_string();
    h.background_color = opt_set(state, "background_color");
    Ok(())
}

fn email_hero_to_form(h: &EmailHero) -> EditFormState {
    let mut st = EditFormState::new(&editform::EMAIL_HERO_FORM);
    st.set("image_src", h.image_src.clone());
    st.set("image_alt", h.image_alt.clone());
    st.set("heading", h.heading.clone());
    st.set("subheading", h.subheading.clone());
    st.set("background_color", opt_get(&h.background_color));
    st
}

fn apply_email_hero(h: &mut EmailHero, state: &EditFormState) -> Result<()> {
    h.image_src = state.get("image_src").trim().to_string();
    h.image_alt = state.get("image_alt").to_string();
    h.heading = state.get("heading").to_string();
    h.subheading = state.get("subheading").to_string();
    h.background_color = opt_set(state, "background_color");
    Ok(())
}

fn email_cta_to_form(c: &EmailCta) -> EditFormState {
    let mut st = EditFormState::new(&editform::EMAIL_CTA_FORM);
    st.set("heading", c.heading.clone());
    st.set("copy", c.copy.clone());
    st.set("button_label", c.button_label.clone());
    st.set("button_href", c.button_href.clone());
    st.set("background_color", opt_get(&c.background_color));
    st
}

fn apply_email_cta(c: &mut EmailCta, state: &EditFormState) -> Result<()> {
    c.heading = state.get("heading").to_string();
    c.copy = state.get("copy").to_string();
    c.button_label = state.get("button_label").to_string();
    c.button_href = state.get("button_href").to_string();
    c.background_color = opt_set(state, "background_color");
    Ok(())
}

fn email_article_to_form(a: &EmailArticle) -> EditFormState {
    let mut st = EditFormState::new(&editform::EMAIL_ARTICLE_FORM);
    st.set("image_src", a.image_src.clone());
    st.set("image_alt", a.image_alt.clone());
    st.set("title", a.title.clone());
    st.set("copy", a.copy.clone());
    st.set("link_label", a.link_label.clone());
    st.set("link_href", a.link_href.clone());
    st.set("image_position", image_pos_str(a.image_position));
    st
}

fn apply_email_article(a: &mut EmailArticle, state: &EditFormState) -> Result<()> {
    a.image_src = state.get("image_src").trim().to_string();
    a.image_alt = state.get("image_alt").to_string();
    a.title = state.get("title").to_string();
    a.copy = state.get("copy").to_string();
    a.link_label = state.get("link_label").to_string();
    a.link_href = state.get("link_href").to_string();
    a.image_position = parse_image_pos(state.get("image_position"))?;
    Ok(())
}

fn email_footer_to_form(f: &EmailFooter) -> EditFormState {
    let mut st = EditFormState::new(&editform::EMAIL_FOOTER_FORM);
    st.set("company_name", f.company_name.clone());
    st.set("address_lines", f.address_lines.join("\n"));
    st.set("unsubscribe_label", f.unsubscribe_label.clone());
    st.set("unsubscribe_href", f.unsubscribe_href.clone());
    st.set("copyright", opt_get(&f.copyright));
    st.sub_state
        .insert("social".into(), social_items(&f.social));
    st.selected_sub_item.insert("social".into(), 0);
    st
}

fn apply_email_footer(f: &mut EmailFooter, state: &EditFormState) -> Result<()> {
    f.company_name = state.get("company_name").to_string();
    f.address_lines = state
        .get("address_lines")
        .split('\n')
        .map(str::to_string)
        .filter(|l| !l.trim().is_empty())
        .collect();
    f.unsubscribe_label = state.get("unsubscribe_label").to_string();
    f.unsubscribe_href = state.get("unsubscribe_href").to_string();
    f.copyright = opt_set(state, "copyright");
    f.social = parse_social_items(state.sub_state.get("social"))?;
    Ok(())
}

fn social_items(elements: &[MjSocialElement]) -> Vec<EditFormState> {
    let mut items = Vec::new();
    for el in elements {
        let mut item = EditFormState::new(&editform::SOCIAL_ITEM_FORM);
        item.set("name", social_net_str(el.name));
        item.set("href", el.href.clone());
        item.set("src", opt_get(&el.src));
        item.set("alt", opt_get(&el.alt));
        item.set("background_color", opt_get(&el.background_color));
        item.set("icon_size", opt_get(&el.icon_size));
        item.set("padding", opt_get(&el.padding));
        item.set("css_class", opt_get(&el.css_class));
        items.push(item);
    }
    items
}

fn parse_social_items(items: Option<&Vec<EditFormState>>) -> Result<Vec<MjSocialElement>> {
    let mut out = Vec::new();
    if let Some(items) = items {
        for item in items {
            out.push(MjSocialElement {
                name: parse_social_net(item.get("name"))?,
                href: item.get("href").to_string(),
                src: opt_set(item, "src"),
                alt: opt_set(item, "alt"),
                background_color: opt_set(item, "background_color"),
                icon_size: opt_set(item, "icon_size"),
                padding: opt_set_named_padding(item, "padding")?,
                css_class: opt_set(item, "css_class"),
            });
        }
    }
    Ok(out)
}

fn opt_get(v: &Option<String>) -> String {
    v.clone().unwrap_or_default()
}

fn sides_to_form(v: &Option<String>) -> String {
    crate::border::Sides::from_storage(v.as_deref().unwrap_or(""))
        .map(|s| s.to_form())
        .unwrap_or_else(|_| "all".to_string())
}

fn opt_set_sides(state: &EditFormState, id: &str) -> Result<Option<String>> {
    let t = state.get(id).trim();
    let sides = crate::border::Sides::from_storage(t).map_err(|e| anyhow!("{id} {e}"))?;
    Ok(sides.to_storage())
}

fn opt_set(state: &EditFormState, id: &str) -> Option<String> {
    let t = state.get(id).trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn opt_set_padding(state: &EditFormState) -> Result<Option<String>> {
    opt_set_named_padding(state, "padding")
}

fn opt_set_named_padding(state: &EditFormState, id: &str) -> Result<Option<String>> {
    let t = state.get(id).trim();
    if t.is_empty() {
        return Ok(None);
    }
    match crate::padding::normalize_padding(t) {
        Ok(v) => Ok(Some(v)),
        Err(msg) => Err(anyhow!("{id} {msg}")),
    }
}

fn opt_set_unit(state: &EditFormState, id: &str) -> Result<Option<String>> {
    let t = state.get(id).trim();
    if t.is_empty() {
        return Ok(None);
    }
    match crate::padding::normalize_padding(t).or_else(|_| crate::padding::normalize_unit(t)) {
        Ok(v) => Ok(Some(v)),
        Err(msg) => Err(anyhow!("{id} {msg}")),
    }
}

fn bool_str(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

fn parse_bool(s: &str) -> Result<bool> {
    match s.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(anyhow!("expected true/false, got {other}")),
    }
}

fn align_str(a: Option<Align>) -> &'static str {
    match a {
        Some(Align::Left) => "left",
        Some(Align::Center) => "center",
        Some(Align::Right) => "right",
        None => "",
    }
}

fn parse_align(s: &str) -> Result<Option<Align>> {
    match s.trim() {
        "" => Ok(None),
        "left" => Ok(Some(Align::Left)),
        "center" => Ok(Some(Align::Center)),
        "right" => Ok(Some(Align::Right)),
        other => Err(anyhow!("invalid align: {other}")),
    }
}

fn hero_mode_str(m: HeroMode) -> &'static str {
    match m {
        HeroMode::FluidHeight => "fluid-height",
        HeroMode::FixedHeight => "fixed-height",
    }
}

fn parse_hero_mode(s: &str) -> Result<HeroMode> {
    match s.trim() {
        "fluid-height" => Ok(HeroMode::FluidHeight),
        "fixed-height" => Ok(HeroMode::FixedHeight),
        other => Err(anyhow!("invalid hero mode: {other}")),
    }
}

fn social_mode_str(m: SocialMode) -> &'static str {
    match m {
        SocialMode::Horizontal => "horizontal",
        SocialMode::Vertical => "vertical",
    }
}

fn parse_social_mode(s: &str) -> Result<SocialMode> {
    match s.trim() {
        "horizontal" => Ok(SocialMode::Horizontal),
        "vertical" => Ok(SocialMode::Vertical),
        other => Err(anyhow!("invalid social mode: {other}")),
    }
}

fn image_pos_str(p: ImagePosition) -> &'static str {
    match p {
        ImagePosition::Top => "top",
        ImagePosition::Left => "left",
        ImagePosition::Right => "right",
    }
}

fn parse_image_pos(s: &str) -> Result<ImagePosition> {
    match s.trim() {
        "top" => Ok(ImagePosition::Top),
        "left" => Ok(ImagePosition::Left),
        "right" => Ok(ImagePosition::Right),
        other => Err(anyhow!("invalid image_position: {other}")),
    }
}

fn thumbnails_str(t: Thumbnails) -> &'static str {
    match t {
        Thumbnails::Visible => "visible",
        Thumbnails::Hidden => "hidden",
        Thumbnails::Supported => "supported",
    }
}

fn parse_thumbnails(s: &str) -> Result<Thumbnails> {
    match s.trim() {
        "visible" => Ok(Thumbnails::Visible),
        "hidden" => Ok(Thumbnails::Hidden),
        "supported" => Ok(Thumbnails::Supported),
        other => Err(anyhow!("invalid thumbnails: {other}")),
    }
}

fn social_net_str(n: SocialNetwork) -> &'static str {
    match n {
        SocialNetwork::Facebook => "facebook",
        SocialNetwork::Instagram => "instagram",
        SocialNetwork::Linkedin => "linkedin",
        SocialNetwork::X => "x",
        SocialNetwork::Github => "github",
        SocialNetwork::Web => "web",
        SocialNetwork::Youtube => "youtube",
        SocialNetwork::Pinterest => "pinterest",
        SocialNetwork::Google => "google",
        SocialNetwork::Tumblr => "tumblr",
        SocialNetwork::Snapchat => "snapchat",
        SocialNetwork::Vimeo => "vimeo",
        SocialNetwork::Medium => "medium",
        SocialNetwork::Soundcloud => "soundcloud",
        SocialNetwork::Dribbble => "dribbble",
        SocialNetwork::Xing => "xing",
    }
}

fn parse_social_net(s: &str) -> Result<SocialNetwork> {
    match s.trim() {
        "facebook" => Ok(SocialNetwork::Facebook),
        "instagram" => Ok(SocialNetwork::Instagram),
        "linkedin" => Ok(SocialNetwork::Linkedin),
        "x" => Ok(SocialNetwork::X),
        "github" => Ok(SocialNetwork::Github),
        "web" => Ok(SocialNetwork::Web),
        "youtube" => Ok(SocialNetwork::Youtube),
        "pinterest" => Ok(SocialNetwork::Pinterest),
        "google" => Ok(SocialNetwork::Google),
        "tumblr" => Ok(SocialNetwork::Tumblr),
        "snapchat" => Ok(SocialNetwork::Snapchat),
        "vimeo" => Ok(SocialNetwork::Vimeo),
        "medium" => Ok(SocialNetwork::Medium),
        "soundcloud" => Ok(SocialNetwork::Soundcloud),
        "dribbble" => Ok(SocialNetwork::Dribbble),
        "xing" => Ok(SocialNetwork::Xing),
        other => Err(anyhow!("invalid social network: {other}")),
    }
}
