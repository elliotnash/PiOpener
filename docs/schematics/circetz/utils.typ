#import "dependencies.typ": cetz
#import cetz: vector
#import cetz.draw: anchor

#let defer-rotation(v, v1, v2) = {
  (
    (v, v1, v2) => {
      // panic(v)
  // panic(v1, v2)
      vector-rotate(v, -get-angle(v1, v2))
    },
    v,
    v1,
    v2
  )
}

#let anchors(anchors) = {
  for (k, v) in anchors {
    anchor(k, v)
  }
}

// Polyfill until next cetz release
#let copy-anchors(element, filter: auto) = {
  (ctx => {
    assert(
      ctx.groups.len() > 0,
      message: "copy-anchors cannot be used outside of a group.",
    )
    assert(
      element in ctx.nodes,
      message: "copy-anchors: Could not find element '" + element + "'",
    )

    let calc-anchors = ctx.nodes.at(element).anchors
    let anchors = calc-anchors(())
    if filter != auto {
      anchors = anchors.filter(a => a in filter)
    }

    // Add each anchor as own element
    for anchor in anchors {
      ctx.nodes.insert(
        anchor,
        (anchors: name => {
          if name == "default" {
            calc-anchors(anchor)
          } else if name == () {
            ("default",)
          } else {
            calc-anchors((anchor,) + name)
          }
        })
      )
      ctx.groups.last().push(anchor)
    }

    return (ctx: ctx)
  },)
}