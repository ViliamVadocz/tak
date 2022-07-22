use crate::search::possible_patterns;

/// Get the number of actually possible moves on each board size.
/// This is used by the older network which has each move one-hot encoded in a
/// vector.
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

/// Get the number of channels needed to encode each move type.
/// This is used by the newer networks.
pub const fn move_channels(n: usize) -> usize {
    let place_types = 3;
    let patterns = possible_patterns(n);
    let spreads = 4 * patterns;
    place_types + spreads
}

/// Multiple number of move channels by board size to get
/// the total size of the network output.
pub const fn output_size(n: usize) -> usize {
    n * n * move_channels(n)
}
