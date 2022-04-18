# AlphaTak

## About

An implementation of [AlphaZero] for the abstract strategy board game [Tak].

The goal of this project is to advance the state of the art in Tak bots,
increase the skill level of the game,
and to experiment with the Rust Machine Learning ecosystem.

I used [tch-rs] crate (wrapper for libtorch) as my machine learning library.
I experimented with [tensorflow] as well, but I found the documentation lacking in many aspects.
Overall, the quality of resources for machine learning in Rust is very poor.

[AlphaZero]: https://deepmind.com/blog/article/alphazero-shedding-new-light-grand-games-chess-shogi-and-go
[Tak]: https://en.wikipedia.org/wiki/Tak_(game)
[tch-rs]: https://github.com/LaurentMazare/tch-rs
[tensorflow]: https://github.com/tensorflow/rust

## Building

It is strongly recommended that you manually install the C++ library PyTorch (libtorch) v1.11.0.
The instructions for installing are can be found [here](https://github.com/LaurentMazare/tch-rs#getting-started).

Then you can just run `cargo build --release` in the `bot` workspace. This will build all the packages in the workspace.
If you want to build a specific one instead, you can specify like this: `cargo build --release -p <package-name>`.

Brief summary of what each package does:

- `tak` library: implementation of Tak including move generation and parsing of PTN and TPS
- `alpha-tak` library: implementation of AlphaZero, MCTS, the network
- `train` binary: training the network with self-play
- `analysis` binary: interactive local analysis
- `playtak` binary: for running the bot on [playtak](https://www.playtak.com/)
