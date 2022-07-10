# bruto

Bruto is an artificial opponent to play [Quarto](https://en.wikipedia.org/wiki/Quarto_(board_game)).

## Quickstart

Type `swap` and hit ENTER after starting the program. This tells the engine to make the first move, and will let you see an example move and display the board.

## Text-based User Interface

The prompt `player>` is displayed when it's your turn to enter a _command_ or a _move_.

The following commands are supported:

- `exit` - exit the program
- `swap` - switch sides with the opponent
- `play <ENGINE_NAME>` - select an engine as opponent
    - `bruto` - (the default) engine based on [MCTS](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search)
    - `rando` - engine that plays random moves

Any input that isn't a valid command is assumed to be a move.

A move is encoded by a sequence of 6 characters of the form `RCPPPP`, where `R` stands for a lowercase character `a..d` encoding a row, `C` stands for a digit `1..4` encoding a column, and `P` is either `o` or `x` encoding a choice for a binary property.

The `RC` part of the move encodes the spot where the piece chosen in the previous turn is to be placed. In the first move of a game, there is no chosen piece yet, which is represented by `RC`=`..`.

The `PPPP` part of the move encodes the piece which the opponent must place in the next turn. If the current move results in an end state (win/loss/draw), there is no such piece, which is represented by `PPPP`=`....`.
