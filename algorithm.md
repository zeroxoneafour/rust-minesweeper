# the minesweeper algorithm
In `src/game.rs`, there is a function called `minesweeper`. This is the first time I have ever made up an algorithm and used it in my code. (Obviously, I'm not the first one to have thought of this algorithm on my own, though). Here is what it does (in procedural order) -
* Defines vecs to return and a vec as an "edge"
* For each coord in edge -
  * If the coord is "clear" (not next to any bombs), it adds all the coords in a "+" space around it to the edge
* Moves all the old, used edge coords to the return vec
* Loops until edge is empty (ie. no new edges can be found)
After edge is finally completely empty, it returns the return vec. This vec has all the tiles to reveal inside of it.

I know that this algorithm is trivial to think of in a few seconds (I thought of it whilst taking a shit), but I'm still proud of myself.
