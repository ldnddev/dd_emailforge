---
component: mj-image
version: 1
node_scope: column_child
insert:
  defaults:
    src: "https://dummyimage.com/600x200/cccccc/000000"
    alt: "Image"
    fluid_on_mobile: true
fields:
  - id: src
    required: true
    type: string
    maps_to: "mj-image src (rewritten in preview/export)"
  - id: alt
    required: true
    type: string
    maps_to: "mj-image alt"
  - id: href
    required: false
    type: string
    maps_to: "mj-image href"
  - id: title
    required: false
    type: string
    maps_to: "mj-image title"
  - id: width
    required: false
    type: string
    maps_to: "mj-image width"
  - id: height
    required: false
    type: string
    hint: "px, %, or auto"
    maps_to: "mj-image height"
  - id: align
    required: false
    type: enum
    options: ["left", "center", "right"]
    maps_to: "mj-image align"
  - id: fluid_on_mobile
    required: false
    type: bool
    default: true
    maps_to: "mj-image fluid-on-mobile=true"
  - id: border
    required: false
    type: string
    hint: "CSS border, e.g. 1px solid #000"
    maps_to: "mj-image border (or border-top/right/bottom/left when sides is a subset)"
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
    maps_to: "mj-image border-radius"
  - id: padding
    required: false
    type: string
    hint: "1-4 values with px or %"
    example: "10px 20px"
    maps_to: "mj-image padding"
---
