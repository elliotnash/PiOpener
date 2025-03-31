#import "../component.typ": component
#import "../dependencies.typ": cetz

#import cetz.draw: *

#let node = (coord, name, p: "tr", d: 0.15) => {
  group(ctx => {
    set-origin(coord)

    // For dot
    circle((0, 0), radius: 0.05)
    
    // For label
    let label-size = cetz.util.measure(ctx, name)
    let x = 0
    let y = 0
    if (p.contains("t")) {
      y = d + label-size.at(1)/2
    } else if (p.contains("b")) {
      y = -d - + label-size.at(1)/2
    } 
    if (p.contains("r")) {
      x = d + label-size.at(0)/2
    } else if (p.contains("l")) {
      x = -d - label-size.at(0)/2
    }
    content((x, y), name, align: top)
  })
}
