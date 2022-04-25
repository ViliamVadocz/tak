use crate::search::possible_patterns;

pub const fn possible_moves(n: usize) -> usize {
    match n {
        3 => 2 * 3 * 3 + 108,   // 126
        4 => 2 * 4 * 4 + 448,   // 480
        5 => 3 * 5 * 5 + 1500,  // 1575
        6 => 3 * 6 * 6 + 4464,  // 4572
        7 => 3 * 7 * 7 + 12348, // 12495
        8 => 3 * 8 * 8 + 32512, // 32704
        _ => unimplemented!(),
    }
}

pub const fn output_size(n: usize) -> usize {
    let place_types = 3;
    let patterns = possible_patterns(n);
    let spreads = 4 * patterns;
    let channels = place_types + spreads;
    n * n * channels
}
