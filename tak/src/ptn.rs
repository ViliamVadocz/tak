use arrayvec::ArrayVec;
use regex::Regex;

use crate::{
    colour::Colour,
    direction::Direction,
    game::{default_starting_stones, Game},
    pos::Pos,
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
            _ => Err(format!("unknown direction {s}")),
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
            .map_err(|_| format!("couldn't parse vertical position {s}"))?
            - 1;
        if x >= N || y >= N {
            return Err(format!("position x={x} y={y} is out of bounds"));
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
            _ => Err(format!("unknown shape {s}")),
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

impl ToPTN for Colour {
    fn to_ptn(&self) -> String {
        match self {
            Colour::White => '1',
            Colour::Black => '2',
        }
        .to_string()
    }
}

impl FromPTN for Colour {
    fn from_ptn(s: &str) -> StrResult<Self> {
        match s {
            "1" => Ok(Colour::White),
            "2" => Ok(Colour::Black),
            _ => Err(format!("unknown colour {s}")),
        }
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
                drop_counts.push(carry_amount);
            }
            if carry_amount != drop_counts.iter().sum() {
                return Err(format!(
                    "picked up {carry_amount} and tried dropping {drop_counts:?} which does not match"
                ));
            }

            let mut moves = ArrayVec::new();
            for drops in drop_counts {
                for _ in 0..(drops - 1) {
                    moves.push(false);
                }
                moves.push(true);
            }
            let last = moves.last_mut().unwrap();
            *last = false;

            Ok(Turn::Move {
                pos,
                direction,
                moves,
            })
        } else {
            let cap = TURN_PLACE_RE
                .captures(s)
                .ok_or_else(|| format!("didn't recognize ply {s}"))?;
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
                    if current > 1 && moves.len() != current - 1 {
                        drops.push_str(&(current - 1).to_string());
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
                        return Err(format!("game size mismatch {value}"));
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
            let turn = Turn::from_ptn(ply)?;
            self.play(turn)?;
        }
        Ok(())
    }
}
