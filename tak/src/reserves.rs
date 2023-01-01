#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reserves<const N: usize> {
    pub(crate) stones: u8,
    pub(crate) caps: u8,
}

impl Default for Reserves<3> {
    fn default() -> Self {
        Self {
            stones: 10,
            caps: 0,
        }
    }
}

impl Default for Reserves<4> {
    fn default() -> Self {
        Self {
            stones: 15,
            caps: 0,
        }
    }
}

impl Default for Reserves<5> {
    fn default() -> Self {
        Self {
            stones: 21,
            caps: 1,
        }
    }
}

impl Default for Reserves<6> {
    fn default() -> Self {
        Self {
            stones: 30,
            caps: 1,
        }
    }
}

impl Default for Reserves<7> {
    fn default() -> Self {
        Self {
            stones: 40,
            caps: 2,
        }
    }
}

impl Default for Reserves<8> {
    fn default() -> Self {
        Self {
            stones: 50,
            caps: 2,
        }
    }
}

impl<const N: usize> Reserves<N> {
    pub const fn depleted(self) -> bool {
        self.caps == 0 && self.stones == 0
    }
}
