use std::fmt::{Display, Write};

use tak::*;

use crate::search::{MoveInfo, Node};

const MAX_BRANCH_LENGTH: usize = 10;
const BRANCH_MIN_VISITS: u32 = 10_000;
const CANDIDATE_MOVE_RATIO: f32 = 0.9;

#[derive(Clone, Debug, Default)]
pub struct Analysis {
    settings: String,
    start_ply: u16,
    played_moves: Vec<Move>,
    move_info: Vec<Option<MoveInfo>>,
    branches: Vec<(u16, MoveInfo)>,
    evals: Vec<f32>,
    marks: Vec<(u16, Mark)>,
}

impl Analysis {
    pub fn new(board_size: u8, half_komi: i8, start_ply: u16) -> Self {
        let settings = format!(
            "[Size \"{}\"]\n[Komi \"{}\"]\n",
            board_size,
            (half_komi / 2).to_string() + if half_komi % 2 == 0 { "" } else { ".5" }
        );

        Analysis {
            settings,
            start_ply,
            ..Default::default()
        }
    }

    pub fn add_setting<T: Display>(&mut self, name: &str, value: T) {
        writeln!(self.settings, "[{name} \"{value}\"]").unwrap();
    }

    pub fn add_move_without_info(&mut self, mov: Move) {
        self.played_moves.push(mov);
        self.move_info.push(None);
    }

    fn add_move(&mut self, mov: Move, info: MoveInfo, eval: f32) {
        self.played_moves.push(mov);
        self.move_info.push(Some(info));
        self.evals.push(eval);
    }

    pub fn update(&mut self, node: &Node, played_move: Move) {
        let debug_info = node.debug(MAX_BRANCH_LENGTH);

        let ply = self.start_ply + self.played_moves.len() as u16;

        let top_visits = debug_info
            .0
            .first()
            .map(|move_info| move_info.visits)
            .unwrap_or_default();
        let eval = debug_info.eval();

        if let Some(prev) = self.evals.last() {
            let eval_diff = -(eval + prev); // due to flipping perspectives
            if (..=-0.5).contains(&eval_diff) {
                self.marks.push((ply - 1, Mark::Blunder))
            } else if (-0.5..=-0.2).contains(&eval_diff) {
                self.marks.push((ply - 1, Mark::Mistake))
            } else if (0.1..=0.3).contains(&eval_diff) {
                self.marks.push((ply - 1, Mark::Strong))
            } else if (0.3..).contains(&eval_diff) {
                self.marks.push((ply - 1, Mark::Brilliancy))
            }
        }

        for info in debug_info.0 {
            // Add info for played move.
            if info.mov == played_move {
                self.add_move(played_move, info, eval);
                continue;
            }

            // Create branches for candidate moves.
            if info.visits as f32 > top_visits as f32 * CANDIDATE_MOVE_RATIO {
                self.branches.push((ply, info));
            }
        }
    }

    pub fn without_branches(mut self) -> Self {
        self.branches = Vec::new();
        self
    }
}

impl Display for Analysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = self.settings.clone();

        let mut move_iter = self.played_moves.iter();
        let mut info_iter = self.move_info.iter();
        let mut eval_iter = self.evals.iter();
        let mut mark_iter = self.marks.iter().peekable();

        let mut ply = self.start_ply;

        // Helper closures.
        let write_eval = |out: &mut String, eval, flip| {
            write!(
                out,
                "{{evaluation: {:+.3}}}",
                eval * if flip { -1.0 } else { 1.0 }
            )
            .unwrap();
        };
        let move_num = |ply| ply / 2 + 1;

        eval_iter.next(); // Consume first eval, so we get the eval after the move is played.

        // Handle starting from black.
        if self.start_ply % 2 != 0 {
            write!(out, "{}. --", move_num(ply)).unwrap();
            // Maybe add black move.
            if let Some(black) = move_iter.next() {
                out.push_str(&black.to_string());
                if let Some((mark_ply, _)) = mark_iter.peek() {
                    if ply == *mark_ply {
                        out.push_str(&mark_iter.next().unwrap().1.to_string());
                    }
                }

                // Maybe add eval.
                if let Some(Some(info)) = info_iter.next() {
                    if let Some(eval) = eval_iter.next() {
                        write_eval(&mut out, eval, false);
                    }
                    out.push_str(&info.ptn_comment(true));
                }
            }
        }

        while let Some(white) = move_iter.next() {
            write!(out, "{}. ", move_num(ply)).unwrap();

            // Add white move.
            out.push_str(&white.to_string());
            if let Some((mark_ply, _)) = mark_iter.peek() {
                if ply == *mark_ply {
                    out.push_str(&mark_iter.next().unwrap().1.to_string());
                }
            }
            // Maybe add eval.
            if let Some(Some(info)) = info_iter.next() {
                if let Some(eval) = eval_iter.next() {
                    write_eval(&mut out, eval, true);
                }
                out.push_str(&info.ptn_comment(false));
            }

            out.push(' ');
            ply += 1;

            // Maybe add black move.
            if let Some(black) = move_iter.next() {
                out.push_str(&black.to_string());
                if let Some((mark_ply, _)) = mark_iter.peek() {
                    if ply == *mark_ply {
                        out.push_str(&mark_iter.next().unwrap().1.to_string());
                    }
                }

                // Maybe add eval.
                if let Some(Some(info)) = info_iter.next() {
                    if let Some(eval) = eval_iter.next() {
                        write_eval(&mut out, eval, false);
                    }
                    out.push_str(&info.ptn_comment(true));
                }
            }
            out.push('\n');
            ply += 1;
        }

        for (ply, branch) in self.branches.iter() {
            // Empty line before branch.
            out.push('\n');
            out.push_str(&format_branch(*ply, branch));
        }

        write!(f, "{out}")
    }
}

fn format_branch(ply: u16, info: &MoveInfo) -> String {
    let mut out = format!("{{{}_{}}}\n", ply, info.mov);

    let mut move_iter = info
        .continuation
        .iter()
        .filter(|(_mov, visits)| visits > &BRANCH_MIN_VISITS)
        .map(|(mov, _visits)| mov.to_string());
    let mut move_num = 1 + ply / 2;

    // First move includes eval comment so it is handled differently.
    if ply % 2 == 0 {
        writeln!(
            out,
            "{move_num}. {} {} {}",
            info.mov,
            info.ptn_comment(false),
            move_iter.next().unwrap_or_default(),
        )
        .unwrap();
    } else {
        writeln!(out, "{move_num}. -- {} {}", info.mov, info.ptn_comment(true)).unwrap();
    }
    move_num += 1;

    // Add the rest of the turns.
    while let Some(white) = move_iter.next() {
        writeln!(
            out,
            "{move_num}. {white} {}",
            move_iter.next().unwrap_or_default()
        )
        .unwrap();
        move_num += 1;
    }

    out
}

#[derive(Clone, Copy, Debug)]
enum Mark {
    Blunder,
    Mistake,
    Strong,
    Brilliancy,
}

impl ToString for Mark {
    fn to_string(&self) -> String {
        match self {
            Mark::Blunder => "??",
            Mark::Mistake => "?",
            Mark::Strong => "!",
            Mark::Brilliancy => "!!",
        }
        .to_string()
    }
}
