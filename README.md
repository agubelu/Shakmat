# Shakmat

Shakmat (approx. transliteration of Russian шахмат, *checkmate*) is a chess engine with a built-in JSON API to interact with.

**Current strength level:** Grandmaster, ~2600 elo (if chess.com's bot ratings are accurate)

Forever a work in progress. Never build a chess engine, it's a bottomless pit :')

## Project structure
Shakmat as a project is composed of several sub-crates that can be compiled independently:
- **Shakmat-core:** Implements the chess board, movement generation and validation, and other related core utilities such as FEN encoding/decoding and Zobrist hashes (library crate).
- **Shakmat-engine:** Implements position evaluation, best move searching and opening books (library crate).
- **Shakmat-server:** Runs a web server that listens for requests and acts as a backend to interact with Shakmat core and engine through a REST API (binary crate).
- **Shakmat-wasm:** Provides a WebAssembly interface layer to integrate and run Shakmat locally on web browsers (library crate).

## How to run Shakmat

Simply run `cargo run --release` and Shakmat will start up and listen for requests. As of now, this will compile and run `shakmat-server` since it's the only binary crate in the workspace. You can also choose to be more explicit and run `cargo run --release -p shakmat-server`.

The default port is `8000`, it can be changed by providing the desired port number as an additional command-line argument, for example, `cargo run --release 9000`

Alternatively, you can compile it with `cargo build --release` and just move the generated binary somewhere else and run it.

Keep in mind that, due to the configuration present in `.cargo/config.toml`, Shakmat is compiled by default with `target-cpu=native` to allow as many CPU-specific optimizations as possible. This means that a built binary may not work on another computer with a different CPU unless this compilation flag is disabled.

## How to use Shakmat

Build and run Shakmat as shown above, and it will start listening for requests.

If you just want to play a game against it, you will have to use a front-end. One is being worked on right now.

If you're interested in interacting with it via its API, check out the [endpoints documentation in the Wiki](https://github.com/agubelu/Shakmat/wiki/API-endpoints).

## Why is the compiled binary several megabytes in size?
Because it includes a [Polyglot](http://hgm.nubati.net/book_format.html) opening book, which is 2.5MB big, and lookup tables for rook and bishop moves using [magic bitboards](https://www.chessprogramming.org/Magic_Bitboards), which are around 3.5MB.

