# QOMF - Quite OK Movie Format

previous pixels now refer to pixel value in prevous frames

run of 60 is 1-2 seconds of video

any encoding must be better than 4 bytes/pixel/frame

- remove rgba, 0xff tag becomes free
    - new base frame?
    - 
- remove index? adapt?
    - chance that a pixel goes back to previous value is slim

- pointer to prev values of neighbors?
    - can model movement
    - how to be smaller than just rgb?
    - how to make fast to encode?

  7  6  5  4  3  2  1  0    7  6  5  4  3  2  1  0
---------------------------------------------------
  0  0 |  dr |  dg |  db |     xoffset |   yoffset          [-2,1] diff, [-8,7] offsets
  0  0 |   xoff |   yoff |          dg | ddr | ddb          [-8,7] luma diff, [-2,1] hue diff, [-4,3] offsets
  0  0 |   xoff |   yoff |    dg |    ddr |    ddb          [-2,1] luma diff, [-4,3] hue diff, [-4,3] offsets

  0  0 |d |          off |          dg | ddr | ddb          [-8,7] luma diff, [-2,1] hue diff, dir, [-16,15] offsets
  0  0 | dir |       off |          dg | ddr | ddb          [-8,7] luma diff, [-2,1] hue diff, [0,3] dir, [1,16] offsets

[0,31] -> [-16,-1][1,16]
32            16    16
if off>0 store off - 1
if off >= 0 load off + 1


state per pixel:
    - previous value
    - run (length in encode, remaining in decode)
