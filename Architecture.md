# Architecture

Terminal-UI email template builder. Authors edit one typed `template.json` in a Ratatui TUI; the app emits strict MJML and compiles HTML with the official Node `mjml` CLI (MJML 5). Single binary, no server in production, no database.

Living product spec: `docs/SPEC.md`. Visual contract: `LDNDDEV_TUI_VISUAL_STANDARD.md`. Component fields: `components/*.md`. Design lock: `docs/DESIGN.md`. End-user walkthrough: [tutorial](https://ldnddev.github.io/dd_emailforge/) (`docs/index.html`; screenshots via `docs/capture.sh`).

## Crate layout

```
crates/ldnddev_theme/     in-tree YAML theme load/save + F2 color editor
src/
  main.rs                 clap CLI: init / tui / validate / export / preview / show
  model.rs                Template → brand / head / body.nodes (serde, kebab-case tags)
  padding.rs              MJML padding shorthand (1-4 px/% values; bare numbers → px)
  storage.rs              JSON load/save, path resolve, atomic write, .backup
  validate.rs             structural + images + marketing footer + version + padding
  emit.rs                 Template → MJML (Preview vs Export rewrite of image src)
  mjml.rs                 discover official CLI, one-shot compile, mjml -w
  preview.rs              loopback wrapper (600/320 iframes), /__mtime, /__meta
  starters.rs             four init templates + package.json / .gitignore / images/.gitkeep
  paths.rs                XDG/config dir, library dir, theme candidates
  tui/mod.rs              App, run loop, autosave
  tui/draw.rs             3-line header / master-detail / 1-line footer
  tui/events.rs           keyboard + mouse dispatch
  tui/theme.rs            AppTheme::load via paths::theme_candidates()
  tui/help.rs             F1 / F2 text
  tui/toasts.rs           four-level ToastLevel
  tui/tree.rs             Structure tree build / nav / expand
  tui/details.rs          inspector + 600px ascii map (email fills) + click-to-select
  tui/editform/           FormEdit types + field maps
  tui/modals/             Modal enum, paint, events, FormEdit, pickers
  tui/cursor.rs           tree id → form-state mapping
  tui/component_kind.rs   insert-picker kinds + legal-target table
  tui/insert.rs           splice a kind at the current selection
  tui/edits.rs            undo / delete / duplicate / reorder / columns
  tui/form_textarea.rs    FormEdit textarea layout
  tui/export.rs           TUI p / Shift+E
  tui/util.rs             open_in_browser, list_dir_entries
  tui/tests.rs            send_key integration
```

JSON is the only source of truth. MJML and HTML are emit targets. There is no MJML round-trip import.

## Document

```
Template
├── version: 1
├── name / subject / preheader / lang / base_url
├── brand          font_family, colors, content_width → mj-attributes
├── head           title, breakpoint, fonts[], json_ld, css, css_inline
└── body.nodes[]
    ├── email-* blocks (header, hero, cta, article, footer)
    └── MJML primitives (section / wrapper / hero / group / column / text / navbar / accordion / carousel / …)
```

One JSON file, one folder. Images live in `images/`. Preview compile lives in `.preview/` (gitignored). Official mjml is pinned in the template `package.json` (`mjml ^5.4.0`).

## Pipeline

```
template.json  --emit-->  template.mjml  --mjml-->  template.html
                                |
                                +-- mjml -w --> .preview/template.html
                                      |
                                      +-- loopback wrapper (TUI p / CLI preview)
```

- **Preview emit** rewrites relative image src to `http://127.0.0.1:{port}/images/…`.
- **Export emit** joins relative src with `base_url` (must be `https://`). Dummyimage `https://` URLs in starters are already valid.
- Exported HTML is exactly `mjml` stdout. Wrapper chrome (`/__mtime`, dual iframes) never ships.

## TUI loop

```
loop:
  tick_autosave(now)              # rewrite template.json 2s after a change
  drain_watch_errors()            # mjml -w stderr → modal
  terminal.draw(|f| self.draw(f)) # 3-line header + master/detail + 1-line footer
  if event::poll(100ms):
    handle_event(evt)
    mark_dirty_if_changed()
```

Body is master/detail (Structure tree + Details inspector), not siteforge's Regions/Pages/Layout. Below 48 columns, Structure only.

### Keys

| Key | Action |
|---|---|
| `F1` | Help |
| `F2` | Theme source + sampled tokens |
| `F3` | Validate (modal on errors, toast otherwise) |
| `p` | Preview (mjml -w + loopback wrapper) |
| `Shift+E` | Export `template.mjml` + `template.html` next to the JSON |
| `s` | Save JSON + `.backup` |
| `/` | Insert picker (legal kinds only) |
| `Enter` | FormEdit |
| `d` / `y` / `u` | Delete / duplicate / undo (cap 20) |
| `J` / `K` | Reorder |
| `C` / `V` | Add / remove column (equal-% rebalance) |
| `c` / `v` | Prev / next column |
| `Ctrl+P` | Image picker on src fields (rooted at `images/`) |
| `Ctrl+Q` | Quit (confirm if dirty). Bare `q` does not quit. |

## Validation

`validate_template` — structural. `validate_template_with_root` — also missing local images. `validate_template_for_export` — plus production URL rules.

F3 / export / preview all gate on errors. Warnings (Gmail clip, unused font, marketing unsub heuristic) toast / print but do not block.

Transactional starter ships empty `unsubscribe_label` **and** empty `unsubscribe_href` so F3 is clean. Marketing starters ship label + `*|UNSUB|*`.
