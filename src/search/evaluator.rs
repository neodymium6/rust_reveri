use crate::board::core::Board;

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> i32;
}
