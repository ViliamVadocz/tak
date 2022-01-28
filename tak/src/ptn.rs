use arrayvec::ArrayVec;
use regex::Regex;

use crate::{
    game::{default_starting_stones, Game},
    pos::{Direction, Pos},
    tile::{Shape, Tile},
    turn::Turn,
    StrResult,
};

lazy_static! {
    // (count)(square)(direction)(drop counts)(stone)
    static ref TURN_MOVE_RE: Regex = Regex::new(r"([1-9]*)([a-z][1-9])([<>+-])([1-9]*)").unwrap();
    // (stone)(square)
    static ref TURN_PLACE_RE: Regex = Regex::new(r"([CS]?)([a-z][1-9])").unwrap();
    static ref OPTIONS_RE: Regex = Regex::new(r#"\[(\S+) ["'](.*?)["']\]"#).unwrap();
    static ref COMMENTS_RE: Regex = Regex::new(r"\{.*?\}").unwrap();
    static ref PLY_SPLIT_RE: Regex = Regex::new(r"\s*\d*\. |\s+|1-0|R-0|F-0|0-1|0-R|0-F|1/2-1/2").unwrap();
}

pub trait FromPTN: Sized {
    fn from_ptn(s: &str) -> StrResult<Self>;
}

pub trait ToPTN {
    fn to_ptn(&self) -> String;
}

impl FromPTN for Direction {
    fn from_ptn(s: &str) -> StrResult<Self> {
        match s {
            "<" => Ok(Direction::NegX),
            ">" => Ok(Direction::PosX),
            "+" => Ok(Direction::PosY),
            "-" => Ok(Direction::NegY),
            _ => Err("unknown direction"),
        }
    }
}

impl ToPTN for Direction {
    fn to_ptn(&self) -> String {
        match self {
            Direction::NegX => "<",
            Direction::PosX => ">",
            Direction::PosY => "+",
            Direction::NegY => "-",
        }
        .to_string()
    }
}

impl<const N: usize> FromPTN for Pos<N> {
    fn from_ptn(s: &str) -> StrResult<Self> {
        let x = (s.bytes().next().ok_or("position is too short")? - b'a') as usize;
        let y = s[1..]
            .parse::<usize>()
            .map_err(|_| "couldn't parse vertical position")?
            - 1;
        if x >= N || y >= N {
            return Err("position is out of bounds");
        }
        Ok(Pos { x, y })
    }
}

impl<const N: usize> ToPTN for Pos<N> {
    fn to_ptn(&self) -> String {
        format!("{}{}", (self.x as u8 + b'a') as char, self.y + 1)
    }
}

impl FromPTN for Shape {
    fn from_ptn(s: &str) -> StrResult<Self> {
        match s {
            "C" => Ok(Shape::Capstone),
            "S" => Ok(Shape::Wall),
            "" => Ok(Shape::Flat),
            _ => Err("unknown shape"),
        }
    }
}

impl ToPTN for Shape {
    fn to_ptn(&self) -> String {
        match self {
            Shape::Flat => "",
            Shape::Wall => "S",
            Shape::Capstone => "C",
        }
        .to_string()
    }
}

impl<const N: usize> FromPTN for Turn<N> {
    fn from_ptn(s: &str) -> StrResult<Self> {
        assert!(N < 10); // the drop notation doesn't support N >= 10

        if let Some(cap) = TURN_MOVE_RE.captures(s) {
            let carry_amount = cap[1].parse().unwrap_or(1);
            let pos = Pos::from_ptn(&cap[2])?;
            let direction = Direction::from_ptn(&cap[3])?;

            let mut drop_counts: Vec<_> = cap[4].chars().map(|c| c.to_digit(10).unwrap()).collect();
            if drop_counts.is_empty() {
                drop_counts = vec![carry_amount];
            }
            let mut moves = ArrayVec::new();
            for drops in drop_counts {
                for _ in 0..(drops - 1) {
                    moves.push(false);
                }
                moves.push(true);
            }

            Ok(Turn::Move {
                pos,
                direction,
                moves,
            })
        } else {
            let cap = TURN_PLACE_RE.captures(s).ok_or("didn't recognize place ply")?;
            let shape = Shape::from_ptn(&cap[1])?;
            let pos = Pos::from_ptn(&cap[2])?;
            Ok(Turn::Place { pos, shape })
        }
    }
}

impl<const N: usize> ToPTN for Turn<N> {
    fn to_ptn(&self) -> String {
        match self {
            Turn::Place { pos, shape } => {
                format!("{}{}", shape.to_ptn(), pos.to_ptn())
            }
            Turn::Move {
                pos,
                direction,
                moves,
            } => {
                if moves.len() < 2 {
                    format!("{}{}", pos.to_ptn(), direction.to_ptn())
                } else {
                    let mut drops = String::new();
                    let mut current = 1;
                    for m in moves {
                        if *m {
                            drops.push_str(&current.to_string());
                            current = 1;
                        } else {
                            current += 1;
                        }
                    }
                    format!("{}{}{}{}", moves.len(), pos.to_ptn(), direction.to_ptn(), drops)
                }
            }
        }
    }
}

impl<const N: usize> FromPTN for Game<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn from_ptn(s: &str) -> StrResult<Game<N>> {
        // parse game options
        let mut komi = 0;
        let (mut stones, mut caps) = default_starting_stones(N);
        for option in OPTIONS_RE.captures_iter(s) {
            let key = &option[0];
            let value = &option[1];
            match key {
                "Komi" => komi = value.parse::<i32>().map_err(|_| "cannot parse komi")?,
                "Flats" => stones = value.parse::<u8>().map_err(|_| "cannot parse flats")?,
                "Caps" => caps = value.parse::<u8>().map_err(|_| "cannot parse caps")?,
                "Size" => {
                    if value.parse::<usize>().map_err(|_| "cannot parse size")? != N {
                        return Err("game size mismatch");
                    }
                }
                _ => {}
            }
        }

        // remove comments
        let s = OPTIONS_RE.replace_all(s, "");
        let s = COMMENTS_RE.replace_all(&s, "");

        // get individual plies (split at move numbers, space, and game result)
        let moves = PLY_SPLIT_RE
            .split(&s)
            .filter(|ss| !ss.is_empty())
            .collect::<Vec<_>>();

        let mut game = Game {
            komi,
            white_stones: stones,
            black_stones: stones,
            white_caps: caps,
            black_caps: caps,
            ..Default::default()
        };
        game.play_ptn_moves(&moves)?;
        Ok(game)
    }
}

impl<const N: usize> Game<N> {
    pub fn play_ptn_moves(&mut self, moves: &[&str]) -> StrResult<()>
    where
        [[Option<Tile>; N]; N]: Default,
    {
        for ply in moves {
            println!("{}", ply);
            let turn = Turn::from_ptn(ply)?;
            self.play(turn)?;
            println!("{}", self.board);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ptn::{FromPTN, ToPTN},
        turn::Turn,
        StrResult,
    };

    const PLIES: &[&str] = &[
        "a6", "f6", "Cd4", "Cc4", "Sd3", "Sc3", "d5", "c5", "d5<", "c4+", "d5", "Se5", "b5", "2c5>11*",
        "2d5<11", "a5", "b4", "a5>", "b4+", "b4", "3b5-21", "2e5<", "d4-*", "d4", "e4", "c4", "e4<", "c4>",
        "2d3+", "2d5<11", "5d4-221", "3b4>111", "2d3+11", "3c5>", "f1",
    ];

    #[test]
    fn ptn_consistency() -> StrResult<()> {
        for ply in PLIES {
            let turn = Turn::<6>::from_ptn(ply)?;
            assert_eq!(turn, Turn::from_ptn(&turn.to_ptn())?);
        }
        Ok(())
    }
}
