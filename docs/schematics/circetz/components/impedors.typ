#import "../component.typ": component
#import "../dependencies.typ": cetz

#import cetz.draw: *

#let impedor(..inputs) = component(
  "impedor",
  (style) => {
    let y = style.height / 2
    let x = style.width / 2
    style.stroke.thickness *= style.thickness

    rect((-x, y), (x, -y), stroke: style.stroke)

    anchor("a", (-x, -y))
    anchor("b", (x, y))
  },
  (
    stroke: auto,
    thickness: auto,
    scale: auto,
    width: 0.8,
    height: 0.3,
  ),
  ..inputs
)