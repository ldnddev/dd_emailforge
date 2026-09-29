//! Four `init --from` templates and the files written next to `template.json`.
use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::model::{
    BodyNode, ColumnChild, EmailArticle, EmailCta, EmailFooter, EmailHeader, EmailHero,
    ImagePosition, MjButton, MjColumn, MjSection, MjText, SectionChild, Template, WebFont,
};
use crate::storage;

const DUMMY: &str = "https://dummyimage.com/600x240/0F1114/FFAF46";
const DUMMY_LOGO: &str = "https://dummyimage.com/320x80/0F1114/FFAF46";
const SYSTEM_FONT: &str = "Arial, Helvetica, sans-serif";
const RALEWAY_HREF: &str =
    "https://fonts.googleapis.com/css2?family=Raleway:wght@400;700&display=swap";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum StarterKind {
    #[default]
    Welcome,
    Newsletter,
    Promo,
    Transactional,
}

impl StarterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::Newsletter => "newsletter",
            Self::Promo => "promo",
            Self::Transactional => "transactional",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Welcome => Self::Newsletter,
            Self::Newsletter => Self::Promo,
            Self::Promo => Self::Transactional,
            Self::Transactional => Self::Welcome,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Welcome => Self::Transactional,
            Self::Newsletter => Self::Welcome,
            Self::Promo => Self::Newsletter,
            Self::Transactional => Self::Promo,
        }
    }
}

impl Template {
    pub fn starter(kind: StarterKind, name: &str) -> Self {
        match kind {
            StarterKind::Welcome => welcome(name),
            StarterKind::Newsletter => newsletter(name),
            StarterKind::Promo => promo(name),
            StarterKind::Transactional => transactional(name),
        }
    }
}

/// Create a template folder: `template.json`, `package.json`, `.gitignore`, `images/.gitkeep`.
/// Refuses to overwrite an existing `template.json`.
pub fn init_template_dir(dir: &Path, kind: StarterKind) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let json = dir.join("template.json");
    if json.exists() {
        anyhow::bail!("refusing to overwrite existing {}", json.display());
    }
    let slug = folder_slug(dir);
    let template = Template::starter(kind, &slug);
    storage::save_template(&json, &template)?;

    let pkg = dir.join("package.json");
    fs::write(&pkg, package_json(&slug))?;
    fs::write(dir.join(".gitignore"), TEMPLATE_GITIGNORE)?;
    let images = dir.join("images");
    fs::create_dir_all(&images)?;
    fs::write(images.join(".gitkeep"), "")?;
    Ok(json)
}

/// Printed after a successful `init`. Local `npm install` is optional when
/// `mjml` is already on PATH (global or otherwise).
pub fn init_next_steps(dir: &Path) -> String {
    format!(
        "If mjml is already installed globally (on your PATH), you can skip npm install.\n\
         Otherwise: cd {} && npm install\n\
         Then: dd_emailforge {}",
        dir.display(),
        dir.display()
    )
}

pub fn folder_slug(dir: &Path) -> String {
    let raw = dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "template".into());
    let slug: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "template".into()
    } else {
        slug
    }
}

fn package_json(slug: &str) -> String {
    format!(
        "{{\n  \"name\": \"{slug}\",\n  \"private\": true,\n  \"description\": \"dd_emailforge template — official MJML 5 compiler pin\",\n  \"devDependencies\": {{\n    \"mjml\": \"^5.4.0\"\n  }}\n}}\n"
    )
}

const TEMPLATE_GITIGNORE: &str = ".preview/\nnode_modules/\ntemplate.json.backup\n";

fn base(name: &str, subject: &str, preheader: &str) -> Template {
    let mut t = Template::minimal();
    t.name = name.to_string();
    t.subject = subject.to_string();
    t.preheader = preheader.to_string();
    t.head.title = subject.to_string();
    t.head.json_ld.clear();
    t.head.css.clear();
    t.head.css_inline = false;
    t.brand.font_family = SYSTEM_FONT.into();
    t
}

fn header() -> EmailHeader {
    EmailHeader {
        logo_src: DUMMY_LOGO.into(),
        logo_alt: "Logo".into(),
        logo_href: Some("https://example.com".into()),
        logo_width: "160px".into(),
        background_color: None,
    }
}

fn hero(heading: &str, subheading: &str) -> EmailHero {
    EmailHero {
        image_src: DUMMY.into(),
        image_alt: heading.into(),
        heading: heading.into(),
        subheading: subheading.into(),
        background_color: None,
    }
}

fn cta(heading: &str, copy: &str, label: &str) -> EmailCta {
    EmailCta {
        heading: heading.into(),
        copy: copy.into(),
        button_label: label.into(),
        button_href: "https://example.com".into(),
        background_color: None,
    }
}

fn article(title: &str, copy: &str) -> EmailArticle {
    EmailArticle {
        image_src: DUMMY.into(),
        image_alt: title.into(),
        title: title.into(),
        copy: copy.into(),
        link_label: "Read more".into(),
        link_href: "https://example.com".into(),
        image_position: ImagePosition::Top,
    }
}

fn marketing_footer() -> EmailFooter {
    EmailFooter {
        company_name: "Example Co".into(),
        address_lines: vec!["123 Main St".into(), "Springfield, IL 62701".into()],
        unsubscribe_label: "Unsubscribe".into(),
        unsubscribe_href: "*|UNSUB|*".into(),
        social: Vec::new(),
        copyright: Some("© Example Co".into()),
    }
}

fn transactional_footer() -> EmailFooter {
    EmailFooter {
        company_name: "Example Co".into(),
        address_lines: vec!["123 Main St".into(), "Springfield, IL 62701".into()],
        unsubscribe_label: String::new(),
        unsubscribe_href: String::new(),
        social: Vec::new(),
        copyright: Some("© Example Co".into()),
    }
}

fn text_col(width: &str, content: &str) -> SectionChild {
    SectionChild::MjColumn(MjColumn {
        width: Some(width.into()),
        background_color: None,
        padding: None,
        inner_background_color: None,
        components: vec![ColumnChild::MjText(MjText {
            content: content.into(),
            align: None,
            font_size: None,
            font_family: None,
            color: None,
            padding: None,
            ..Default::default()
        })],
        ..Default::default()
    })
}

fn welcome(name: &str) -> Template {
    let mut t = base(name, "You're in.", "Here's what happens next.");
    t.head.fonts.push(WebFont {
        name: "Raleway".into(),
        href: RALEWAY_HREF.into(),
    });
    t.brand.font_family = "Raleway, Arial, Helvetica, sans-serif".into();
    t.body.nodes = vec![
        BodyNode::EmailHeader(header()),
        BodyNode::EmailHero(hero("You're in.", "Thanks for joining us.")),
        BodyNode::MjSection(MjSection {
            background_color: None,
            padding: None,
            full_width: false,
            children: vec![
                text_col("50%", "What you get: a weekly note, no spam."),
                text_col("50%", "What we need: nothing else. You're set."),
            ],
            ..Default::default()
        }),
        BodyNode::EmailCta(cta(
            "Start here",
            "Open the app and pick a template.",
            "Open the app",
        )),
        BodyNode::EmailFooter(marketing_footer()),
    ];
    t
}

fn newsletter(name: &str) -> Template {
    let mut t = base(name, "This week", "Two reads worth your time.");
    t.body.nodes = vec![
        BodyNode::EmailHeader(header()),
        BodyNode::EmailHero(hero("This week", "A short digest.")),
        BodyNode::EmailArticle(article(
            "Story one",
            "The first piece from this week's roundup.",
        )),
        BodyNode::EmailArticle(article(
            "Story two",
            "The second piece, same length, same dummyimage.",
        )),
        BodyNode::EmailCta(cta(
            "See all",
            "The full archive is on the site.",
            "Archive",
        )),
        BodyNode::EmailFooter(marketing_footer()),
    ];
    t
}

fn promo(name: &str) -> Template {
    let mut t = base(
        name,
        "30% off this week",
        "A short window. That's the point.",
    );
    t.body.nodes = vec![
        BodyNode::EmailHeader(header()),
        BodyNode::EmailHero(hero("30% off", "This week only.")),
        BodyNode::EmailCta(cta(
            "Shop the sale",
            "Discount applies at checkout.",
            "Shop now",
        )),
        BodyNode::EmailArticle(article(
            "Featured product",
            "One item. Dummyimage placeholder. Swap it in the TUI.",
        )),
        BodyNode::EmailFooter(marketing_footer()),
    ];
    t
}

fn transactional(name: &str) -> Template {
    let mut t = base(name, "Your receipt", "Order confirmed.");
    t.body.nodes = vec![
        BodyNode::EmailHeader(header()),
        BodyNode::MjSection(MjSection {
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
                        content: "Thanks — your order is confirmed.".into(),
                        align: None,
                        font_size: None,
                        font_family: None,
                        color: None,
                        padding: None,
                        ..Default::default()
                    }),
                    ColumnChild::MjButton(MjButton {
                        content: "View order".into(),
                        href: "https://example.com/orders/1".into(),
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
        }),
        BodyNode::EmailFooter(transactional_footer()),
    ];
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::validate_template;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("dd_emailforge_init_{}_{}", std::process::id(), n));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn init_writes_expected_files() {
        let dir = temp_dir().join("welcome-email");
        let json = init_template_dir(&dir, StarterKind::Welcome).unwrap();
        assert!(json.is_file());
        assert!(dir.join("package.json").is_file());
        assert!(dir.join(".gitignore").is_file());
        assert!(dir.join("images/.gitkeep").is_file());
        let pkg = fs::read_to_string(dir.join("package.json")).unwrap();
        assert!(pkg.contains("\"mjml\": \"^5.4.0\""));
        assert!(pkg.contains("\"name\": \"welcome-email\""));
        let gi = fs::read_to_string(dir.join(".gitignore")).unwrap();
        assert!(gi.contains(".preview/"));
        assert!(gi.contains("node_modules/"));
        assert!(gi.contains("template.json.backup"));
        let t = storage::load_template(&json).unwrap();
        assert_eq!(t.name, "welcome-email");
        assert_eq!(t.version, 1);
        assert!(t.head.json_ld.is_empty());
        assert!(t.head.css.is_empty());
        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn init_next_steps_mentions_global_mjml() {
        let msg = init_next_steps(Path::new("welcome-email"));
        assert!(msg.contains("already installed globally"));
        assert!(msg.contains("skip npm install"));
        assert!(msg.contains("cd welcome-email && npm install"));
        assert!(msg.contains("dd_emailforge welcome-email"));
    }

    #[test]
    fn init_refuses_overwrite() {
        let dir = temp_dir().join("exists");
        init_template_dir(&dir, StarterKind::Welcome).unwrap();
        let err = init_template_dir(&dir, StarterKind::Promo).unwrap_err();
        assert!(err.to_string().contains("refusing to overwrite"));
        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }

    #[test]
    fn all_starters_validate() {
        for kind in [
            StarterKind::Welcome,
            StarterKind::Newsletter,
            StarterKind::Promo,
            StarterKind::Transactional,
        ] {
            let t = Template::starter(kind, kind.as_str());
            let report = validate_template(&t);
            assert!(
                report.errors.is_empty(),
                "{} errors: {:?}",
                kind.as_str(),
                report.errors
            );
        }
    }

    #[test]
    fn transactional_is_f3_clean() {
        let t = Template::starter(StarterKind::Transactional, "receipt");
        let report = validate_template(&t);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        match t.body.nodes.last() {
            Some(BodyNode::EmailFooter(f)) => {
                assert!(f.unsubscribe_label.is_empty());
                assert!(f.unsubscribe_href.is_empty());
                assert!(!f.address_lines.is_empty());
            }
            other => panic!("expected footer, got {other:?}"),
        }
    }

    #[test]
    fn welcome_cleared_unsub_href_warns() {
        let mut t = Template::starter(StarterKind::Welcome, "welcome");
        for node in &mut t.body.nodes {
            if let BodyNode::EmailFooter(f) = node {
                f.unsubscribe_href.clear();
            }
        }
        let report = validate_template(&t);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(
            report.warnings.iter().any(|w| w.contains("unsubscribe")),
            "{:?}",
            report.warnings
        );
    }

    #[test]
    fn welcome_uses_raleway() {
        let t = Template::starter(StarterKind::Welcome, "welcome");
        assert_eq!(t.head.fonts.len(), 1);
        assert_eq!(t.head.fonts[0].name, "Raleway");
        assert!(t.brand.font_family.contains("Raleway"));
        let n = Template::starter(StarterKind::Newsletter, "n");
        assert!(n.head.fonts.is_empty());
        assert_eq!(n.brand.font_family, SYSTEM_FONT);
    }

    #[test]
    fn starters_round_trip() {
        for kind in [
            StarterKind::Welcome,
            StarterKind::Newsletter,
            StarterKind::Promo,
            StarterKind::Transactional,
        ] {
            let t = Template::starter(kind, kind.as_str());
            let json = serde_json::to_string(&t).unwrap();
            let back: Template = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back, "{}", kind.as_str());
        }
    }

    #[test]
    fn starter_node_shapes() {
        let w = Template::starter(StarterKind::Welcome, "w");
        assert!(matches!(w.body.nodes[0], BodyNode::EmailHeader(_)));
        assert!(matches!(w.body.nodes[1], BodyNode::EmailHero(_)));
        assert!(matches!(w.body.nodes[2], BodyNode::MjSection(_)));
        assert!(matches!(w.body.nodes[3], BodyNode::EmailCta(_)));
        assert!(matches!(w.body.nodes[4], BodyNode::EmailFooter(_)));
        if let BodyNode::MjSection(s) = &w.body.nodes[2] {
            assert_eq!(s.children.len(), 2);
        }

        let n = Template::starter(StarterKind::Newsletter, "n");
        assert_eq!(n.body.nodes.len(), 6);
        assert!(matches!(n.body.nodes[2], BodyNode::EmailArticle(_)));
        assert!(matches!(n.body.nodes[3], BodyNode::EmailArticle(_)));

        let p = Template::starter(StarterKind::Promo, "p");
        if let BodyNode::EmailHero(h) = &p.body.nodes[1] {
            assert!(h.heading.contains("30%"));
        }

        let tx = Template::starter(StarterKind::Transactional, "t");
        assert!(matches!(tx.body.nodes[1], BodyNode::MjSection(_)));
        assert_eq!(tx.body.nodes.len(), 3);
    }

    #[test]
    fn welcome_emit_has_font_and_no_include() {
        let t = Template::starter(StarterKind::Welcome, "welcome");
        let mjml = crate::emit::emit_mjml(&t, crate::emit::EmitMode::Export).unwrap();
        assert!(mjml.contains("mj-font"));
        assert!(mjml.contains("Raleway"));
        assert!(!mjml.contains("mj-include"));
        assert!(mjml.contains("mj-preview"));
        assert!(mjml.contains("mj-attributes"));
    }

    #[test]
    #[ignore = "requires official mjml CLI"]
    fn mjml_strict_compiles_welcome() {
        let dir = temp_dir().join("welcome");
        let json = init_template_dir(&dir, StarterKind::Welcome).unwrap();
        let t = storage::load_template(&json).unwrap();
        let mjml_path = dir.join("template.mjml");
        crate::emit::write_mjml(&t, &mjml_path, crate::emit::EmitMode::Export).unwrap();
        let bin = crate::mjml::discover_mjml(&dir).expect("mjml on PATH");
        let out = dir.join("template.html");
        crate::mjml::compile_one_shot_captured(&bin, &dir, &mjml_path, &out).unwrap();
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.to_ascii_lowercase().contains("<!doctype html") || html.contains("<html"));
        assert!(!crate::preview::html_contains_wrapper_markers(&html));
        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }
}
