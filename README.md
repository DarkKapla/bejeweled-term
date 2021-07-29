# Bejeweled-term

A match-the-three game in the terminal interface.  

Written in Rust and powered by the ncurses library.

### How to play

Navigate through the tiles with the arrow keys, and press
Z/Q/S/D to swap the current gem with its neighbor.

#### Bugs

Doing simply "endwin(); panic!();" will discard the panic message
(it's eaten in the terminal because NCurses releases the "screen mode" afterward).
