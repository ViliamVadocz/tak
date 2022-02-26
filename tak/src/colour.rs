use std::{str::FromStr, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

impl FromStr for Colour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        if lower == "w" || lower == "white" {
            Ok(Colour::White)
        } else if lower == "b" || lower == "black" {
            Ok(Colour::Black)
        } else {
            Err(format!("could not convert {s} to colour"))
        }
    }
}

impl Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
