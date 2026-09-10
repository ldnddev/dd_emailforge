---
component: mj-wrapper
version: 1
node_scope: body_node
insert:
  defaults:
    full_width: false
    children: []
fields:
  - id: label
    required: false
    type: string
    hint: "Tree and blueprint only — not in the email"
    example: "footer"
    maps_to: "TUI author label (not emitted)"
  - id: background_color
    required: false
    type: string
    maps_to: "mj-wrapper background-color"
  - id: background_url
    required: false
    type: url
    maps_to: "mj-wrapper background-url"
  - id: background_size
    required: false
    type: string
    maps_to: "mj-wrapper background-size"
  - id: background_repeat
    required: false
    type: enum
    options: ["repeat", "no-repeat"]
    maps_to: "mj-wrapper background-repeat"
  - id: gap
    required: false
    type: string
    hint: "px or %"
    maps_to: "mj-wrapper gap"
  - id: padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "10px 20px"
    maps_to: "mj-wrapper padding"
  - id: border
    required: false
    type: string
    hint: "CSS border, e.g. 1px solid #000"
    maps_to: "mj-wrapper border (or border-top/right/bottom/left when sides is a subset)"
  - id: border_sides
    required: false
    type: string
    hint: "all, or a list of top, right, bottom, left"
    default: all
    maps_to: "omitted → border=; subset → border-{side}="
  - id: border_radius
    required: false
    type: string
    hint: "px or %"
    example: "8px"
    maps_to: "mj-wrapper border-radius"
  - id: full_width
    required: false
    type: bool
    default: false
    maps_to: "mj-wrapper full-width=full-width"
  - id: children
    required: true
    type: list
    maps_to: "mj-section | mj-hero only"
---
