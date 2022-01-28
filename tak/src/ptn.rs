use regex::Regex;

use crate::{
    game::{default_starting_stones, Game},
    pos::{Direction, Pos},
    tile::Tile,
    turn::Turn,
    StrResult,
};

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

impl<const N: usize> FromPTN for Turn<N> {
    fn from_ptn(s: &str) -> StrResult<Self> {
        todo!()
    }

    // FIXME once Turn is board agnostic
    // fn from_ptn(ply: &str, board: &Board<N>, colour: Colour) ->
    // StrResult<Turn<N>> { assert!(N < 10);
    // (count)(square)(direction)(drop counts)(stone)
    // let re = Regex::new(r"([0-9]*)([a-z])([0-9])([<>+-])([0-9]*)[CS]?").unwrap();
    // if let Some(cap) = re.captures(ply) {
    // let carry_amount = cap[1].parse().unwrap_or(1);
    //
    // let x = abc_to_num(cap[2].chars().next().unwrap());
    // let y = cap[3].parse::<usize>().unwrap() - 1;
    // let direction = Direction::from_ptn(&cap[4]);
    //
    // let mut drop_counts: Vec<_> = cap[5].chars().map(|c|
    // c.to_digit(10).unwrap()).collect(); if drop_counts.is_empty() {
    // drop_counts = vec![carry_amount];
    // }
    //
    // let mut pos = Pos { x, y };
    // let tile = board[pos].clone().ok_or("there is not stack on that position")?;
    // let (_left, mut carry) = tile.take::<N>(carry_amount as usize)?;
    //
    // let mut drops = ArrayVec::new();
    // for i in drop_counts {
    // pos = pos.step(direction).ok_or("move would go off the board")?;
    // for _ in 0..i {
    // drops.push((
    // pos,
    // carry
    // .pop()
    // .ok_or("not enough pieces picked up to satisfy move ply")?,
    // ))
    // }
    // }
    //
    // Ok(Turn::Move {
    // pos: Pos { x, y },
    // drops,
    // })
    // } else {
    // (stone)(square)
    // let re = Regex::new(r"([CS]?)([a-z])([0-9])").unwrap();
    // let cap = re.captures(ply).ok_or("didn't recognize place ply")?;
    // let shape = Shape::from_ptn(&cap[1]);
    // let x = abc_to_num(cap[2].chars().next().unwrap());
    // let y = cap[3].parse::<usize>().unwrap() - 1;
    //
    // Ok(Turn::Place {
    // pos: Pos { x, y },
    // piece: Piece { shape, colour },
    // })
    // }
    // }
}

impl<const N: usize> ToPTN for Turn<N> {
    fn to_ptn(&self) -> String {
        match self {
            Turn::Place { pos, piece } => {
                format!("{}{}", piece.shape.to_ptn(), pos.to_ptn())
            }
            Turn::Move { pos, drops } => {
                let mut direction = None;
                let mut spread = String::new();
                let mut current = 1;
                let mut last = pos;
                for (drop, _piece) in drops {
                    if direction.is_none() {
                        direction = Some((*drop - *pos).unwrap());
                    } else if drop == last {
                        current += 1;
                    } else {
                        spread.push_str(&current.to_string());
                        current = 1;
                    }
                    last = drop;
                }
                // leave out drop number if we are moving the whole stack
                if !spread.is_empty() {
                    spread.push_str(&current.to_string());
                }
                if drops.len() > 1 {
                    format!(
                        "{}{}{}{}",
                        drops.len(),
                        pos.to_ptn(),
                        direction.unwrap().to_ptn(),
                        spread
                    )
                } else {
                    format!("{}{}", pos.to_ptn(), direction.unwrap().to_ptn())
                }
            }
        }
    }
}

// TODO lazy_static REGEX

impl<const N: usize> FromPTN for Game<N>
where
    [[Option<Tile>; N]; N]: Default,
{
    fn from_ptn(s: &str) -> StrResult<Game<N>> {
        // parse game options
        let mut komi = 0;
        let (mut stones, mut caps) = default_starting_stones(N);
        let options_re = Regex::new(r#"\[(\S+) ["'](.*?)["']\]"#).unwrap();
        for option in options_re.captures_iter(s) {
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
        let comments_re = Regex::new(r"\{.*?\}").unwrap();
        let s = options_re.replace_all(s, "");
        let s = comments_re.replace_all(&s, "");

        // get individual plies (split at move numbers, space, and game result)
        let re = Regex::new(r"\s\d*. |\s+|1-0|R-0|F-0|0-1|0-R|0-F|1/2-1/2").unwrap();
        let moves = re.split(&s).collect::<Vec<_>>();

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

#[cfg(test)]
mod tests {
    use crate::{
        game::Game,
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
        let mut game = Game::<6>::default();
        for ply in PLIES {
            let turn = Turn::from_ptn(ply)?;
            assert_eq!(turn, Turn::from_ptn(&turn.to_ptn())?);
            game.play(turn)?;
        }
        Ok(())
    }
}
