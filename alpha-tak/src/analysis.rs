use std::fmt::Display;

use tak::*;

use crate::search::{MoveInfo, Node};

const MAX_BRANCH_LENGTH: usize = 10;
const BRANCH_MIN_VISITS: u32 = 100;
const CANDIDATE_MOVE_RATIO: f32 = 0.7;

#[derive(Clone, Debug, Default)]
pub struct Analysis {
    board_size: u8,
    half_komi: i8,
    played_moves: Vec<Move>,
    move_info: Vec<Option<MoveInfo>>,
    branches: Vec<(usize, MoveInfo)>,
}

impl Analysis {
    pub fn new(board_size: u8, half_komi: i8) -> Self {
        Analysis {
            board_size,
            half_komi,
            ..Default::default()
        }
    }

    pub fn add_move_without_info(&mut self, mov: Move) {
        self.played_moves.push(mov);
        self.move_info.push(None);
    }

    fn add_move(&mut self, mov: Move, info: MoveInfo) {
        self.played_moves.push(mov);
        self.move_info.push(Some(info));
    }

    pub fn update(&mut self, node: &Node, played_move: Move) {
        let debug_info = node.debug(MAX_BRANCH_LENGTH);

        let top_visits = debug_info
            .0
            .first()
            .map(|move_info| move_info.visits)
            .unwrap_or_default() as f32;

        let ply = self.played_moves.len();

        for info in debug_info.0 {
            // Add info for played move.
            if info.mov == played_move {
                self.add_move(played_move, info);
                continue;
            }

            // Create branches for candidate moves.
            if info.visits as f32 > top_visits * CANDIDATE_MOVE_RATIO {
                self.branches.push((ply, info));
            }
        }
    }
}

impl Display for Analysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!(
            "[Size \"{}\"]\n[Komi\"{}\"]\n",
            self.board_size,
            (self.half_komi / 2).to_string() + if self.half_komi % 2 == 0 { "" } else { ".5" }
        );

        let mut move_iter = self.played_moves.iter();
        let mut info_iter = self.move_info.iter();
        let mut move_num = 1;
        while let Some(white) = move_iter.next() {
            // Add white move.
            out.push_str(&format!("{move_num}. "));
            out.push_str(&white.to_string());

            // Maybe add eval.
            if let Some(Some(info)) = info_iter.next() {
                out.push_str(&info.ptn_comment(false));
            }
            out.push(' ');

            // Maybe add black move.
            if let Some(black) = move_iter.next() {
                out.push_str(&black.to_string());
                // Maybe add eval.
                if let Some(Some(info)) = info_iter.next() {
                    out.push_str(&info.ptn_comment(true));
                }
            }
            out.push('\n');

            move_num += 1;
        }

        for (ply, branch) in self.branches.iter() {
            // Empty line before branch.
            out.push('\n');
            out.push_str(&format_branch(*ply, branch));
        }

        write!(f, "{out}")
    }
}

fn format_branch(ply: usize, info: &MoveInfo) -> String {
    let mut out = format!("{{{}_{}}}\n", ply, info.mov);

    let mut move_iter = info
        .continuation
        .iter()
        .filter(|(_mov, visits)| visits > &BRANCH_MIN_VISITS)
        .map(|(mov, _visits)| mov.to_string());
    let mut move_num = 1 + ply / 2;

    // First move includes eval comment so it is handled differently.
    if ply % 2 == 0 {
        out.push_str(&format!(
            "{move_num}. {} {} {}\n",
            info.mov,
            info.ptn_comment(false),
            move_iter.next().unwrap_or_default(),
        ));
    } else {
        out.push_str(&format!(
            "{move_num}. -- {} {}\n",
            move_iter.next().unwrap(),
            info.ptn_comment(true),
        ));
    }
    move_num += 1;

    // Add the rest of the turns.
    while let Some(white) = move_iter.next() {
        out.push_str(&format!(
            "{move_num}. {white} {}\n",
            move_iter.next().unwrap_or_default()
        ));
        move_num += 1;
    }

    out
}
