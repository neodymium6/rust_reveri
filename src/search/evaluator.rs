use crate::board::core::Board;

pub trait Evaluator: Send + Sync {
    fn evaluate(&self, board: &Board) -> i32;
}

#[derive(Clone)]
pub struct PieceEvaluator {}
impl Evaluator for PieceEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        board.diff_piece_num()
    }
}

#[derive(Clone)]
pub struct LegalNumEvaluator {}
impl Evaluator for LegalNumEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        board.get_legal_moves_vec().len() as i32
    }
}
