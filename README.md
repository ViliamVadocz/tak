# AlphaTak

## About

An implementation of [AlphaZero] for the abstract strategy board game [Tak].

The goal of this project is to advance the state of the art in Tak bots
and to experiment with the Rust Machine Learning ecosystem.

I used [tch-rs] crate (wrapper for libtorch) as my machine learning library.
I experimented with [tensorflow] as well, but I found the documentation lacking in many aspects.
Overall, the quality of resources for machine learning in Rust is very poor.

[AlphaZero]: https://deepmind.com/blog/article/alphazero-shedding-new-light-grand-games-chess-shogi-and-go
[Tak]: https://en.wikipedia.org/wiki/Tak_(game)
[tch-rs]: https://github.com/LaurentMazare/tch-rs
[tensorflow]: https://github.com/tensorflow/rust
