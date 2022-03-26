A collection of ideas and things I want to implement so I dont forget

# Must do
- Endgame tablebases/bitbases
- Convert non-castling castling moves from the opening book
- Better evaluation for pawns (structure, passed...)
- Configuration endpoint
- Multithreading (will probably be very painful)
- Better focus on PV and better pruning
- Efficient move generator for checks (must keep track of pins, checkers...)

# Ideas
- Put midgame and endgame position tables together using i16 pairs
- Check if unmake move is faster than copy
- Web interface updated in real time for the comparison script
- Check if using bitboards to detect if a move is a capture is faster than the array access
- Check if `const fn` improves something for things like check detection
- https://chessboardjs.com
- https://rich.readthedocs.io/en/stable/introduction.html for the python script (so overkill :D)
- Maybe UCI integration
- Look at Rust profilers
- Experiment with variable panic times depending on depth/score drop
- Set a compile flag to disable useless/debug prints