





#set text(font:"JetBrainsMono NF", weight: 700, size: 65pt)
#set page(width: 100pt, height: 100pt, fill: none, margin: (x: 0pt, y: 0pt))

#let x = 16pt
#let y = 26pt

#box(fill: white, width: 100pt, height: 100pt, radius: 50pt)[
  #place(top + left, 
  move(dx: x, dy: y)[W]
)

#place(top + left, 
move(dx: x + 34pt, dy: y, text(fill:rgb(30, 64, 175), style: "italic")[B])
  )
]
