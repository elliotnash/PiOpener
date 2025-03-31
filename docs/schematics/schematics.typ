#import "@preview/cetz:0.3.4": canvas, draw
#import "/circetz/lib.typ": components, circuits
#import "@preview/cetz-plot:0.1.1"

#let fg-color = white

#set page(
  width: auto,
  height: auto,
  margin: 1em,
  fill: black
)

#set text(fill: fg-color)

// Limit switch voltage divider

#canvas({
  import components: *
  import draw: *
  set-style(
    stroke: (thickness: 0.4pt, paint: fg-color),
    fill: fg-color,
    circetz: (
      style: (
        current: "american",
        voltage: "american",
      ),
    )
  )

  node((0,0), p: "l")[Bottom Limit Switch +]
  node((0,2), p: "l")[Top Limit Switch +]

  resistor((0,0), (3,0), l: $1 M Omega$)
  resistor((), (6,0), l: $670 k Omega$)

  resistor((0,2), (3,2), l: $1 M Omega$)
  resistor((), (6,2), l: $670 k Omega$)

  short((), (6,0))

  short((6,2), (6.5,2))
  node((), p: "r")[GND PIN]

  {
    rotate(90deg)
    ground((0,-6))
    rotate(-90deg)
  }

  short((3,2), (3,3), poles: "*-")
  node((), p: "t")[GPIO 21]
  
  short((3,0), (3,-1), poles: "*-")
  node((), p: "b")[GPIO 22]
})

#pagebreak()

// Optoisolator circuit (button presser)

#canvas({
  import components: *
  import draw: *
  set-style(
    stroke: (thickness: 0.4pt, paint: fg-color),
    fill: fg-color,
    circetz: (
      style: (
        current: "american",
        voltage: "american"
      ),
    )
  )

  flipflop((), name: "ic", l: "PC817", t: (
      "1": [An],
      "3": [Cat],
      "6": [Col],
      "4": [Em],
    ),
  )

  node((rel: (0.28,0), to: "ic.north-west"), "")

  short("ic.pin 6", (rel: (0.5,0), to: "ic.pin 6"))
  node((), p: "r")[Garage Button +]
  
  short("ic.pin 4", (rel: (0.5,0), to: "ic.pin 4"))
  node((), p: "r")[Garage Button -]

  resistor("ic.pin 1", (rel: (-2,0), to: "ic.pin 1"), l: $330 Omega$)
  node((), p: "l")[GPIO 17]

  short("ic.pin 3", (rel: (-2,0), to: "ic.pin 3"))
  node((), p: "l")[GND]
})
