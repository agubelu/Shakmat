A collection of ideas and things I want to implement so I dont forget

# Must do
- Experiment: don't enter a new iteration if the time for the last one > time remaining
- Endgame tablebases
- Search struct to store info during the search
- Null move
- Time management
- Convert non-castling castling moves from the opening book
- Better evaluation for pawns (structure, passed...)
- Configuration endpoint
- Multithreading (will probably be very painful)
- Better focus on PV and better pruning
- Efficient move generator for checks (must keep track of pins, checkers...)

# Ideas
- Check if unmake move is faster than copy
- Web interface updated in real time for the comparison script
- Check if using bitboards to detect if a move is a capture is faster than the array access
- Check if `const fn` improves something for things like check detection
- https://chessboardjs.com
- https://rich.readthedocs.io/en/stable/introduction.html for the python script (so overkill :D)
- Maybe UCI integration
- Look at Rust profilers
- Experiment with variable panic times depending on depth/score drop