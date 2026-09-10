---
component: mj-column
version: 1
node_scope: section_child
insert:
  defaults:
    components: []
fields:
  - id: width
    required: false
    type: string
    maps_to: "mj-column width"
  - id: background_color
    required: false
    type: string
    maps_to: "mj-column background-color"
  - id: padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "10px 20px"
    maps_to: "mj-column padding"
  - id: inner_background_color
    required: false
    type: string
    maps_to: "mj-column inner-background-color"
  - id: border
    required: false
    type: string
    hint: "CSS border, e.g. 1px solid #000"
    maps_to: "mj-column border (or border-top/right/bottom/left when sides is a subset)"
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
    maps_to: "mj-column border-radius"
  - id: inner_border
    required: false
    type: string
    maps_to: "mj-column inner-border (or inner-border-top/right/bottom/left when sides is a subset)"
  - id: inner_border_sides
    required: false
    type: string
    hint: "all, or a list of top, right, bottom, left"
    default: all
    maps_to: "omitted → inner-border=; subset → inner-border-{side}="
  - id: inner_border_radius
    required: false
    type: string
    hint: "px or %"
    maps_to: "mj-column inner-border-radius"
  - id: vertical_align
    required: false
    type: enum
    options: ["top", "middle", "bottom"]
    maps_to: "mj-column vertical-align"
  - id: components
    required: true
    type: list
    maps_to: "column children (mj-text, mj-button, mj-image, mj-divider, mj-spacer, mj-social, mj-table, mj-navbar, mj-accordion, mj-carousel)"
---
