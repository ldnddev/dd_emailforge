# dd_emailforge — Spec

Living product spec. Update this file when behavior or conventions change.

Companion docs:

- `Architecture.md` — crate map, pipeline, keys
- `LDNDDEV_TUI_VISUAL_STANDARD.md` — portable TUI theme + shell contract
- `components/*.md` — per-component fields, emit rules, validation
- `docs/DESIGN.md` — locked design (do not silently fork)
- `docs/tutorial/index.html` — setup / install / TUI walkthrough with screenshots
- `README.md` — short install + link to the tutorial

---

## Product

Terminal-UI email template builder. Single Rust binary. Author edits a typed `template.json`, exports strict MJML, compiles HTML with official MJML 5.

Target: CRM / designers / developers shipping one email at a time. No campaign container, no shared `brand.json`, no `mj-include`.

Workflow:

1. `init <dir> [--from welcome|newsletter|promo|transactional]`
2. `cd <dir> && npm install` if `mjml` is not already on PATH (official `mjml ^5.4.0`, Node 20+)
3. `tui <dir>` — Structure tree + Details + FormEdit
4. `p` preview (loopback wrapper, 600px + 320px)
5. `export` writes `template.mjml` + `template.html` next to the JSON

Current crate version: see `Cargo.toml`. JSON `version` is `1` and independent of crate semver.

---

## Shipped surface

### CLI

```
dd_emailforge init <dir> [--from welcome|newsletter|promo|transactional]
dd_emailforge tui [template.json|dir]
dd_emailforge validate <template.json|dir>
dd_emailforge export <template.json|dir> [--out dir]
dd_emailforge preview <template.json|dir> [--port 8766]
dd_emailforge show <template.json>
```

A directory argument means `dir/template.json`. `init` does **not** run `npm install`; it prints the hint. CLI `preview` binds port **8766** by default; TUI preview binds `127.0.0.1:0`.

### Content model

`Template` → `brand` + `head` + `body.nodes`. Nodes are internally tagged kebab-case (`email-header`, `mj-section`, …). Nested enums are kebab-case (`fluid-height`, `horizontal`, `top`, JSON `"x"` for Twitter, `hidden` for carousel thumbnails).

Insert is parent-scoped: `mj-section` children are `mj-column` / `mj-group` (leaves splice into the last column). `mj-column` / `mj-hero` take `ColumnChild` (`mj-text` … `mj-navbar` / `mj-accordion` / `mj-carousel`). `mj-navbar-link` only under `mj-navbar`; `mj-accordion-element` only under `mj-accordion`; `mj-carousel-image` only under `mj-carousel`. `mj-wrapper` is `mj-section` / `mj-hero`. Email-* blocks are body-level only. `mj-table.content` is row markup (`<tr>` / `<td>` / `<th>`), not a wrapping `<table>` — `mj-table` already emits one. `inner-padding` is legal on `mj-button`, `mj-social`, and `mj-hero`. On `mj-button`, a pixel `width` smaller than inner-padding left+right (+ border) is a validate error — MJML would collapse the button to 0px.

New fields take `#[serde(default)]`. Unknown JSON `version` is refused at load (exit 2). Never silently coerce `version: 2`.

### TUI

Fixed 3-line header + master/detail body + 1-line adaptive footer per the visual standard.

Toasts: `success` / `info` / `warning` / `error`. Modals for blocking errors (load, missing mjml, strict compile fail, validation list, FormEdit, pickers).

F1 Help. F2 Theme. F3 Validate. `Shift+E` Export. `p` Preview. `s` Save. `/` insert (filtered to legal kinds). `Ctrl+Q` quit (confirm if dirty). Bare `q` never quits.

FormEdit: Tab / enum cycle / Ctrl+S / Esc / click-to-focus / Ctrl+P image picker. Padding fields show the MJML rule (1-4 values with `px` or `%`, e.g. `10px` or `10px 20px`); empty inputs show that example as a placeholder. Bare numbers are saved and emitted as `px`. Invalid units (`em`, five values, …) fail FormEdit save and F3. P0 chrome on primitives: `border` / `border_radius` (section, column, wrapper, image, button, hero, social, carousel, table), typography (`font_weight` / `font_style` / `line_height`), button `inner_padding`, divider `border_style`/`width`/`align`. P1 layout: section `gutter` + background-url/size/repeat/direction, wrapper background-url/size/repeat/`gap`, column/group/hero `vertical_align`, group `direction`, hero `background_width`/`background_position`, button `target`/`height`, image `title`, social icon/inner padding + font-size/color, document `dir`. P2 completeness: `css_class` on body + primitives, extra type/link attrs, navbar hamburger `ico-*`, accordion icons, carousel thumbnail radius, extra social networks, table cellpadding/cellspacing/role. Empty optional attrs are omitted; brand `mj-attributes` stay the fallback. Tree: `j/k` `g/G` `h/l` Space, `d` `y` `u`, `J/K`, `C/V` `c/v`. Details shows a full-email ascii blueprint of every layout node, filled with each node's resolved email colors (inherit brand → body → ancestors); the selected element is chrome-highlighted (`text_active_focus` on glyphs, fill stays); click a region to select it in the Structure tree.

### Starters

| Kind | Footer |
|---|---|
| welcome, newsletter, promo | address + Unsubscribe + `*|UNSUB|*` |
| transactional | address; **empty** unsub label **and** href (F3 clean) |

Welcome may ship one Google Font (Raleway) and set `brand.font_family` to use it. Others stay on `Arial, Helvetica, sans-serif`. No JSON-LD / custom CSS in starters. Dummyimage `https://` placeholders are allowed.

### Compiler

Official Node `mjml` CLI only. Discover `{root}/node_modules/.bin/mjml` then `PATH`. Never mrml, never `npx`. Flags: `validationLevel=strict`, beautify, keepComments, sanitizeStyles, allowMixedSyntax, templateSyntax JSON array. **No** `allowIncludes`.

---

## Non-goals

- MJML or HTML import / round-trip
- Shared `brand.json` / `mj-include` / `allowIncludes`
- In-TUI screenshot preview
- Inventing TUI tokens (promote them in the visual standard first)
- A template library UI (the dir is created on install, unused in v1)

---

## Conventions

### Branch + commit

Feature work on `feat/<short-name>` off `main`. Commits use plain prefixes: `tui:`, `model:`, `validate:`, `docs:`, `test:`, or a PR-slice title (`PR 6`). Tags: annotated `vMAJOR.MINOR.PATCH`. Do not re-tag.

### Tests

`#[cfg(test)] mod tests` at the bottom of the module. Drive TUI via in-tree `send_key`, not by poking state when a key path exists. `cargo test -q`. Official mjml is **not** required on CI; compiler-backed tests are `#[ignore]`.

### New component

1. Spec in `components/<name>.md`
2. Type in `src/model.rs`
3. Emitter in `src/emit.rs`
4. FormEdit map in `src/tui/editform/`
5. Insert kind + legal-target row in `src/tui/component_kind.rs`

### New modal

Four-point plumbing: enum variant + render dispatch + event dispatch + a `Modal::…` match arm.

### Theme

Always `self.theme.*`. Canonical tokens only. Labels: `text_labels` → `text_active_focus` when focused. Folders/files/links in pickers. Modal section headers: `modal_header` (bold). Lookup: `./dd_emailforge_theme.yml`, then `$XDG_CONFIG_HOME/ldnddev/` else `~/.config/ldnddev/`.

### UX prefs

- Footer is a 1-line adaptive key bar. Always starts with `F1:Help`. No compile status.
- Quit is `Ctrl+Q` only.
- Browser launch pins stdio to `/dev/null`.
- Preview binds loopback only.

---

## Anti-patterns

- Features that were not requested
- Compiling with mrml
- Emitting `mj-include` or passing `--config.allowIncludes`
- Shipping wrapper livereload in exported HTML
- Invented theme tokens (`email_preview`, `mjml_tag`, `canvas`, …)
- Bare `q` as quit
