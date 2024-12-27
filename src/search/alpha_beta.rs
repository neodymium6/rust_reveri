use crate::board::core::Board;
use crate::search::evaluator::Evaluator;
use crate::search::time_keeper::TimeKeeper;
use crate::search::transposition_table::{EntryType, TranspositionTable};

pub struct AlphaBetaSearch {
    max_depth: usize,
    evaluator: Box<dyn Evaluator>,
    tt: TranspositionTable,
}

impl AlphaBetaSearch {
    pub fn new(max_depth: usize, evaluator: Box<dyn Evaluator>) -> Self {
        Self {
            max_depth,
            evaluator,
            tt: TranspositionTable::new(),
        }
    }

    fn tt_cutoff(&self, board: &Board, depth: usize, alpha: i32, beta: i32) -> Option<i32> {
        if let Some(entry) = self.tt.lookup(board) {
            if entry.get_depth() >= depth {
                match entry.get_type() {
                    EntryType::Exact => return Some(entry.get_score()),
                    EntryType::LowerBound => {
                        if entry.get_score() >= beta {
                            return Some(entry.get_score());
                        }
                    }
                    EntryType::UpperBound => {
                        if entry.get_score() <= alpha {
                            return Some(entry.get_score());
                        }
                    }
                }
            }
        }
        None
    }

    fn get_search_score(&mut self, board: &Board, depth: usize, alpha: i32, beta: i32) -> i32 {
        if let Some(score) = self.tt_cutoff(board, depth, alpha, beta) {
            return score;
        }
        if board.is_game_over() {
            match board.is_win() {
                Ok(true) => return i32::MAX - 2,
                Ok(false) => match board.is_lose() {
                    Ok(true) => return i32::MIN + 2,
                    Ok(false) => return 0,
                    Err(_) => return 0,
                },
                Err(_) => return 0,
            }
        }
        if depth == 0 {
            return self.evaluator.evaluate(board);
        }

        let mut current_alpha = alpha;
        if let Some(child_boards) = board.get_child_boards() {
            for child_board in child_boards {
                let score = -self.get_search_score(&child_board, depth - 1, -beta, -current_alpha);
                if score > current_alpha {
                    current_alpha = score;
                }
                if current_alpha >= beta {
                    // cut
                    self.tt
                        .store(board, depth, current_alpha, EntryType::LowerBound);
                    return current_alpha;
                }
            }
            let entry_type = if current_alpha == alpha {
                // not changed -> no better score -> upper bound
                EntryType::UpperBound
            } else {
                // new best score -> exact
                EntryType::Exact
            };
            self.tt.store(board, depth, current_alpha, entry_type);
            current_alpha
        } else {
            // pass
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            let score = -self.get_search_score(&new_board, depth, -beta, -alpha);
            let entry_type = if score <= alpha {
                // cut happened
                EntryType::UpperBound
            } else if score >= beta {
                EntryType::LowerBound
            } else {
                EntryType::Exact
            };
            self.tt.store(board, depth, score, entry_type);
            score
        }
    }

    pub fn get_move(&mut self, board: Board) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for move_i in board.get_legal_moves_vec() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score = -self.get_search_score(&new_board, self.max_depth, -beta, -alpha);
            if score > alpha {
                alpha = score;
                best_move = Some(move_i);
            }
        }
        self.tt
            .store(&board, self.max_depth, alpha, EntryType::Exact);
        best_move
    }

    fn get_search_score_with_timeout(
        &mut self,
        board: &Board,
        depth: usize,
        alpha: i32,
        beta: i32,
        time_keeper: &TimeKeeper,
    ) -> i32 {
        if let Some(score) = self.tt_cutoff(board, depth, alpha, beta) {
            return score;
        }
        if board.is_game_over() {
            match board.is_win() {
                Ok(true) => return i32::MAX - 2,
                Ok(false) => match board.is_lose() {
                    Ok(true) => return i32::MIN + 2,
                    Ok(false) => return 0,
                    Err(_) => return 0,
                },
                Err(_) => return 0,
            }
        }
        if depth == 0 {
            return self.evaluator.evaluate(board);
        }

        let mut current_alpha = alpha;
        if let Some(child_boards) = board.get_child_boards() {
            for child_board in child_boards {
                let score = -self.get_search_score_with_timeout(
                    &child_board,
                    depth - 1,
                    -beta,
                    -current_alpha,
                    time_keeper,
                );
                if score > current_alpha {
                    current_alpha = score;
                }
                if current_alpha >= beta {
                    // cut
                    self.tt
                        .store(board, depth, current_alpha, EntryType::LowerBound);
                    return current_alpha;
                }
                if time_keeper.is_timeout() {
                    return current_alpha;
                }
            }
            let entry_type = if current_alpha == alpha {
                // not changed -> no better score -> upper bound
                EntryType::UpperBound
            } else {
                // new best score -> exact
                EntryType::Exact
            };
            self.tt.store(board, depth, current_alpha, entry_type);
            current_alpha
        } else {
            // pass
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            let score =
                -self.get_search_score_with_timeout(&new_board, depth, -beta, -alpha, time_keeper);
            let entry_type = if score <= alpha {
                // cut happened
                EntryType::UpperBound
            } else if score >= beta {
                EntryType::LowerBound
            } else {
                EntryType::Exact
            };
            self.tt.store(board, depth, score, entry_type);
            score
        }
    }

    fn get_move_with_timeout(
        &mut self,
        board: &Board,
        depth: usize,
        time_keeper: &TimeKeeper,
    ) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for move_i in board.get_legal_moves_vec() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score =
                -self.get_search_score_with_timeout(&new_board, depth, -beta, -alpha, time_keeper);
            if time_keeper.is_timeout() {
                return best_move;
            }
            if score > alpha {
                alpha = score;
                best_move = Some(move_i);
            }
        }
        self.tt.store(board, depth, alpha, EntryType::Exact);
        best_move
    }

    pub fn get_move_with_iter_deepening(
        &mut self,
        board: Board,
        timeout: std::time::Duration,
    ) -> Option<usize> {
        let mut best_move = None;
        let time_keeper = TimeKeeper::new(timeout);
        let mut depth = 0;
        loop {
            let move_i = self.get_move_with_timeout(&board, depth, &time_keeper);
            if time_keeper.is_timeout() {
                break;
            }
            if let Some(m) = move_i {
                best_move = Some(m);
            }
            depth += 1;
        }
        best_move
    }
}
