use tak::*;
use tch::{kind::FLOAT_CPU, Tensor};

fn create_reserves_tensor<const N: usize>(stones: u8, max: u8) -> Tensor {
    let mut reserves = Tensor::zeros(&[max as i64, N as i64, N as i64], FLOAT_CPU);
    if stones > 0 {
        reserves = reserves.index_put_(
            &[Some(Tensor::of_slice(&[(stones - 1) as i64])), None, None],
            &Tensor::ones(&[N as i64, N as i64], FLOAT_CPU),
            false,
        );
    }
    reserves
}

pub fn reserves_repr<const N: usize>(game: &Game<N>) -> (Tensor, Tensor, Tensor, Tensor) {
    let (stones, capstones) = default_starting_stones(N);
    let white_stones = create_reserves_tensor::<N>(game.white_stones, stones);
    let black_stones = create_reserves_tensor::<N>(game.black_stones, stones);
    let white_caps = create_reserves_tensor::<N>(game.white_caps, capstones);
    let black_caps = create_reserves_tensor::<N>(game.black_caps, capstones);

    if game.to_move == Color::White {
        (white_stones, black_stones, white_caps, black_caps)
    } else {
        (black_stones, white_stones, black_caps, white_caps)
    }
}
