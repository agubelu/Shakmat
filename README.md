# Shakmat

Shakmat (approx. transliteration of Russian шахмат, *checkmate*) is a chess engine with a built-in JSON API to interact with.

**Current strength level:** Grandmaster, ~2600 elo (if chess.com's bot ratings are accurate)

Forever a work in progress. Never build a chess engine, it's a bottomless pit :')

## How to run Shakmat

Simply run `cargo run --release` and Shakmat will start up and listen for requests. The default port is `8000`, it can be changed by providing the desired port number as an additional command-line argument, for example, `cargo run --release 9000`

Alternatively, you can compile it with `cargo build --release` and just move the generated binary somewhere else and run it.

Keep in mind that, due to the configuration present in `.cargo/config.toml`, Shakmat is compiled by default with `target-cpu=native` to allow as many CPU-specific optimizations as possible. This means that a built binary may not work on another computer with a different CPU unless this compilation flag is disabled.

## How to use Shakmat

Build and run Shakmat as shown above, and it will start listening for requests.

If you just want to play a game against it, you will have to use a front-end. One is being worked on right now.

If you're interested in interacting with it via its API, check out the [endpoints documentation in the Wiki](https://github.com/agubelu/Shakmat/wiki/API-endpoints).

## Why is the compiled binary several megabytes in size?
Because it includes a [Polyglot](http://hgm.nubati.net/book_format.html) opening book, which is 2.5MB big, and lookup tables for rook and bishop moves using [magic bitboards](https://www.chessprogramming.org/Magic_Bitboards), which are around 3.5MB.

