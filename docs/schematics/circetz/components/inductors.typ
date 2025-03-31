#import "../component.typ": component
#import "../dependencies.typ": cetz

#import cetz.draw: *

#let inductor(..inputs) = component(
  "inductor",
  (style) => {
    let step = style.width / (style.coils * 2) 
    let height = style.height
    let x = style.width / 2
    style.stroke.thickness *= style.thickness

    anchor("x", (-x, 0))
    arc(("x"), start: 180deg, delta: -180deg, radius: step)

    for _ in range(style.coils - 1) {
      arc((), start: 180deg, delta: -180deg, radius: step)
    }

    anchor("a", (-x, -height/2))
    anchor("b", (x, height/2))
  },
  (
    stroke: auto,
    thickness: auto,
    scale: auto,
    width: 0.8,
    height: 0.3,
    coils: 4,
  ),
  ..inputs
)