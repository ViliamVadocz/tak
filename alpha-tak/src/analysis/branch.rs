use arrayvec::ArrayVec;
use tak::*;

use super::{MAX_BRANCH_LENGTH, move_info::MoveInfo};

pub struct Branch<const N: usize> {
    pub ply: usize,
    pub line: ArrayVec<Turn<N>, MAX_BRANCH_LENGTH>,
    pub info: MoveInfo,
}

impl<const N: usize> ToPTN for Branch<N> {
    fn to_ptn(&self) -> String {
        let mut out = format!("{{{}_{}}}\n", self.ply, self.line.first().unwrap().to_ptn());

        let mut turn_iter = self.line.iter().map(|t| t.to_ptn());
        let mut move_num = 1 + self.ply / 2;

        // first move includes eval comment so it is handled differently
        if self.ply % 2 == 0 {
            out.push_str(&format!(
                "{move_num}. {} {{{}}} {}\n",
                turn_iter.next().unwrap(),
                self.info.to_ptn(),
                turn_iter.next().unwrap_or_default()
            ));
        } else {
            out.push_str(&format!(
                "{move_num}. -- {} {{{}}}\n",
                turn_iter.next().unwrap(),
                self.info.to_ptn()
            ));
        }
        move_num += 1;

        // add the rest of the turns
        while let Some(white) = turn_iter.next() {
            out.push_str(&format!(
                "{move_num}. {white} {}\n",
                turn_iter.next().unwrap_or_default()
            ));
            move_num += 1;
        }

        out
    }
}
