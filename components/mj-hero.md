---
component: mj-hero
version: 1
node_scope: body_node
insert:
  defaults:
    mode: fluid-height
    children: []
fields:
  - id: mode
    required: false
    type: enum
    options: ["fluid-height", "fixed-height"]
    maps_to: "mj-hero mode"
  - id: background_url
    required: false
    type: string
    maps_to: "mj-hero background-url (same rewrite as mj-image src)"
  - id: background_color
    required: false
    type: string
    maps_to: "mj-hero background-color"
  - id: background_height
    required: false
    type: string
    maps_to: "mj-hero background-height"
  - id: background_width
    required: false
    type: string
    maps_to: "mj-hero background-width"
  - id: background_position
    required: false
    type: string
    maps_to: "mj-hero background-position"
  - id: width
    required: false
    type: string
    maps_to: "mj-hero width"
  - id: height
    required: false
    type: string
    maps_to: "mj-hero height"
  - id: padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    maps_to: "mj-hero padding"
  - id: inner_padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "12px 24px"
    maps_to: "mj-hero inner-padding"
  - id: border_radius
    required: false
    type: string
    hint: "px or %"
    maps_to: "mj-hero border-radius"
  - id: vertical_align
    required: false
    type: enum
    options: ["top", "middle", "bottom"]
    maps_to: "mj-hero vertical-align"
  - id: children
    required: true
    type: list
    maps_to: "ColumnChild list (hero is a single column)"
---
