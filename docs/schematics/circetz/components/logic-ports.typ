#import "../component.typ": component
#import "../dependencies.typ": cetz

#import cetz.draw: *

#let logic-port(
  name,
  func,
  style,
  ..inputs
) = component(
  ("logic-ports", name),
  style => {
    style.stdH = 0.5 * style.height * style.baselen
    style.pin-length *= style.baselen
    style.not-radius *= style.stdH
    style.left = -8 * style.stdH / 6.5
    anchor("left", (style.left, 0))
    anchor("right", (8 * style.stdH / 6.5, 0))
    style.inputs = calc.max(2, style.number-inputs)

    

    func(style)
  },
  (
    (
      baselen: 0.4,
      pin-length: 0.7,
      height: 2,
      scale: auto,
      stroke: auto,
      fill: white,
      thickness: auto,
      not-radius: 0.154,
      inner-inputs: 0,
      number-inputs: 0
    ),
    style
  ),
  ..inputs
)

#let notcirc = logic-port.with(
  "notcirc", 
  style => {
    style.stroke.thickness *= style.thickness
    circle((), radius: style.not-radius, stroke: style.stroke, fill: style.fill)
  },
  (
    stroke: auto
  )
)

#let and-port = logic-port.with(
  "and-port",
  style => {

    let inner = if style.inner-inputs == 0 {
      style.inputs
    } else {
      style.inner-inputs
    }
    style.pin-distance = style.stdH * 2 / inner

    for i in range(1, style.inputs + 1) {
      anchor("bin " + str(i), (style.left, style.inputs * style.pin-distance * 0.75 - style.pin-distance * i))
      anchor("in " + str(i), (rel: (-style.pin-length, 0)))

      line("bin " + str(i), "in " + str(i))
    }

    anchor("bout", "right")
    anchor("out", (rel: (style.pin-length, 0)))
    line("bout", "out")

    let height = style.stdH
    let left = style.left
    // let up = style.inputs / 2 * style.pin-distance
    // let temp = up + 0.5 * style.pin-distance
    // for _ in range(style.inputs) {
    //   temp -= style.pin-distance
    //   line(
    //     (left, temp),
    //     (left - style.pin-length, temp)
    //   )
    // }


    style.stroke.thickness *= style.thickness
    merge-path({
      line(
        (left, height),
        (1.5 * height / 6.5, height)
      )
      arc((), start: 90deg, stop: -90deg, radius: height)
      line((), 
        (left, -height)
      )
    }, close: true, stroke: style.stroke)

  },
  (
    stroke: auto,
  )
)