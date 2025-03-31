#import "dependencies.typ": cetz,
#import "components.typ": *

#import cetz: canvas, draw
#import draw: *

#let thevenin-eq = (v, r) => {    
  vsource((0,0), (0,3), l: [$V_"th" = $ #v])
  resistor((), (3,3), l: [$R_"th" = $ #r])
  short((0,0), (3,0))

  node((3,3), "A", p: "r")
  node((3,0), "B", p: "r")
}

#let norton-eq = (i, r) => {
  isource((0,0), (0,3), l: [$I_N = $ #i])
  short((), (3,3))
  short((0,0), (3,0))
  resistor((1,3), (1,0), l: [$R_N = $ #r])

  node((3,3), "A", p: "r")
  node((3,0), "B", p: "r")
}
