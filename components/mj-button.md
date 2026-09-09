---
component: mj-button
version: 1
node_scope: column_child
insert:
  defaults:
    content: "Read more"
    href: "https://example.com"
fields:
  - id: content
    required: true
    type: string
    maps_to: "mj-button inner text (plain, XML-escaped)"
  - id: href
    required: true
    type: string
    maps_to: "mj-button href"
    ui:
      note: "Absolute https or an opaque merge tag."
  - id: background_color
    required: false
    type: string
    maps_to: "mj-button background-color"
  - id: color
    required: false
    type: string
    maps_to: "mj-button color"
  - id: align
    required: false
    type: enum
    options: ["left", "center", "right"]
    maps_to: "mj-button align"
  - id: font_family
    required: false
    type: string
    maps_to: "mj-button font-family"
  - id: font_size
    required: false
    type: string
    hint: "px or %"
    maps_to: "mj-button font-size"
  - id: font_weight
    required: false
    type: enum
    options: ["normal", "bold", "400", "700"]
    maps_to: "mj-button font-weight"
  - id: font_style
    required: false
    type: enum
    options: ["normal", "italic"]
    maps_to: "mj-button font-style"
  - id: border
    required: false
    type: string
    hint: "CSS border, e.g. 1px solid #000"
    maps_to: "mj-button border"
  - id: border_radius
    required: false
    type: string
    hint: "px or %"
    maps_to: "mj-button border-radius"
  - id: height
    required: false
    type: string
    hint: "px or %"
    maps_to: "mj-button height"
  - id: target
    required: false
    type: enum
    options: ["_blank", "_self"]
    maps_to: "mj-button target"
  - id: inner_padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "12px 24px"
    maps_to: "mj-button inner-padding (omit → brand default 12px 24px). With a px width, left+right must leave room inside the button or MJML emits width:0."
  - id: width
    required: false
    type: string
    maps_to: "mj-button width"
  - id: padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "10px 20px"
    maps_to: "mj-button padding"
---
