---
component: mj-table
version: 1
node_scope: column_child
insert:
  defaults:
    content: "<tr><td></td></tr>"
fields:
  - id: content
    required: true
    type: string
    ui:
      control: textarea
      hint: "Rows only (<tr>/<td>/<th>). mj-table already emits <table>."
    maps_to: "mj-table inner — <tr>…</tr> rows (no wrapping <table>)"
  - id: font_size
    required: false
    type: string
    maps_to: "mj-table font-size"
  - id: font_family
    required: false
    type: string
    maps_to: "mj-table font-family"
  - id: line_height
    required: false
    type: string
    maps_to: "mj-table line-height"
  - id: color
    required: false
    type: string
    maps_to: "mj-table color"
  - id: align
    required: false
    type: enum
    options: ["left", "center", "right"]
    maps_to: "mj-table align"
  - id: width
    required: false
    type: string
    maps_to: "mj-table width"
  - id: border
    required: false
    type: string
    maps_to: "mj-table border"
  - id: padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "10px 20px"
    maps_to: "mj-table padding"
---
