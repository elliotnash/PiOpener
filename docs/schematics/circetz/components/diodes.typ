#import "../component.typ": component
#import "../dependencies.typ": cetz

#import cetz.draw: *

#let led(..inputs) = component(
  "led",
  (style) => {
    style.stroke.thickness *= style.thickness

    line(
      (-style.width/2, -style.height/2),
      (-style.width/2, style.height/2),
      (style.width/2, 0),
      close: true, 
      name: "t",
      stroke: style.stroke,
      fill: style.fill
    )

    line(
      (style.width/2, -style.height/2),
      (style.width/2, style.height/2),
      stroke: style.stroke,
    )
    
    anchor("a", (style.width/2, style.height/2))
    anchor("b", (-style.width/2, -style.height/2))
    
    fill(black)
    set-style(stroke: (thickness: 1pt))
    line((0.275, 0.6), (0, 0.33), mark: (start: ">", scale: 0.8))
    line((0.475, 0.6), (0.2, 0.33), mark: (start: ">", scale: 0.8))
  },
  (
    stroke: auto,
    thickness: auto,
    fill: auto,
    style: auto,
    scale: auto,
    width: 0.4,
    height: 0.5
  ),
  ..inputs
)
