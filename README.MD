## WroteColor

![](/sample/vision_image.gif) <br>
Wrote Color project is A simple game server in asynchronous Rust
for typing games.

### Project target

It is working on html.. colored block will move left to right.
Block's color will be random(000000 ~ FFFFFF).

Then, player types the block's color.
But don't worry, color will be reference next below..

* Red
* Green
* Black
* Blue
* Yellow

### What we have to make modules?

* Process that block to moving... (left -> right)
* Choose randomly color that set to block
* Synchronized that blocks
* Removed block(entity) when any player has type correctly.
* listening player's answers.

