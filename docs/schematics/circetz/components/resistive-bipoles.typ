#import "../component.typ": component
#import "../dependencies.typ": cetz

#import cetz.draw: *

#let short(..inputs) = component(
  "short",
  (style) => {
    // anchor("a", (0.05, 0.05))
    // anchor("b", (-0.05, -0.05))
  },
  (
    stroke: auto
  ),
  ..inputs
)

#let resistor(..inputs) = component(
  "resistor",
  (style) => {
    let step = style.width / (style.zigs * 2)
    let height = style.height
    let sgn = -1
    let x = style.width / 2
    style.stroke.thickness *= style.thickness
    line(
      (-x, 0),
      (rel: (step/2, height/2)),
      ..for _ in range(style.zigs * 2 - 1) {
        ((rel: (step, height * sgn)),)
        sgn *= -1
      },
      (x, 0),
      fill: none,
      stroke: style.stroke
    )
    anchor("a", (-x, -height/2))
    anchor("b", (x, height/2))
    // anchor("text", (0, style.label.last()))
  },
  (
    stroke: auto,
    thickness: auto,
    scale: auto,
    width: 0.8,
    height: 0.3,
    zigs: 3,
  ),
  ..inputs
)

#let photocell(..inputs) = component(
  "photocell",
  (style) => {
    style.stroke.thickness *= style.thickness
    circle(
      (0, 0),
      radius: (style.width/2, style.height/2),
      name: "c",
      stroke: style.stroke,
      fill: style.fill
    )
    anchor("a", ("c.west", "|-", "c.south"))
    anchor("b", ("c.east", "|-", "c.north"))
    
    resistor("c.east", "c.west", scale: 0.4)

    fill(black)
    set-style(stroke: (thickness: 1pt))
    line((0.275, 0.6), (0, 0.33), mark: (end: ">", scale: 0.8))
    line((0.475, 0.6), (0.2, 0.33), mark: (end: ">", scale: 0.8))
  },
  (
    stroke: auto,
    thickness: auto,
    fill: auto,
    style: auto,
    scale: auto,
    width: 0.6,
    height: 0.6
  ),
  ..inputs
)
