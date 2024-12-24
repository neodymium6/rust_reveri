use crate::search::evaluator::Evaluator;

pub struct AlphaBetaSearch {
    max_depth: usize,
    evaluator: Box<dyn Evaluator>,
}
