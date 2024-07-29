# Devlog

## Next steps

- How to model blocks that are part of the Matrix?
  - it needs to be easy enough to scan for patterns (i.e whole lines to remove)
  - should it be just plain `Block` components with a `Pos` component that are
  _not_ children of the current `Piece`?
    - That might need to a lot of sorting to look for patterns, check for lock
    phase, etc...
- Turn the current piece into plain blocks after lock phase

## 2023-06-13

- Turns out an entity's `transform` is relative to its parent. Which makes a lot
of sense.
  - Reworked things so that blocks are now 1.0 unit in size, and there is a
  global scale factor that is applied to the Matrix entity to make everything as
  big as needed.
  - Blocks are children of the piece they belong to. The current piece is a
  child of the Matrix entity (i.e the board/field).
- Drew a "floor" below the first row (i.e y=-1), and make sure pieces don't fall
below that.

## 2023-06-11

- Got a little skeleton going.
- Modeled the different tetriminoes and their different orientations, only
taking into account the "visual rotation center" for now.
- Added a dummy "splash screen" just because.
- Moved the game logic into its own plugin.
