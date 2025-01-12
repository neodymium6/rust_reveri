use crate::board::{self, core::Board};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone)]
pub struct TTEntry {
    depth: usize,
    score: i32,
    entry_type: EntryType,
}

impl TTEntry {
    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn get_type(&self) -> EntryType {
        self.entry_type
    }
}

pub struct TranspositionTable {
    table: HashMap<Board, TTEntry>,
    max_size: usize,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            max_size: 60_000,
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn store(&mut self, board: &Board, depth: usize, score: i32, entry_type: EntryType) {
        if self.table.len() >= self.max_size {
            self.cleanup_early_entries(board, depth);
        }

        if let Some(entry) = self.lookup(board) {
            if entry.get_depth() > depth {
                return;
            }
            if entry.get_depth() == depth {
                match (entry.get_type(), entry_type) {
                    (EntryType::Exact, _) => return,
                    (_, EntryType::Exact) => (),
                    (EntryType::LowerBound, EntryType::LowerBound) => {
                        if entry.get_score() >= score {
                            return;
                        }
                    }
                    (EntryType::UpperBound, EntryType::UpperBound) => {
                        if entry.get_score() <= score {
                            return;
                        }
                    }
                    _ => (),
                }
            }
        }
        self.table.insert(
            board.clone(),
            TTEntry {
                depth,
                score,
                entry_type,
            },
        );
    }

    pub fn lookup(&self, board: &Board) -> Option<&TTEntry> {
        self.table.get(board)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    fn cleanup_early_entries(&mut self, board: &Board, depth: usize) {
        let boards_to_remove: Vec<Board> = self
            .table
            .keys()
            .filter(|b| b.piece_sum() < board.piece_sum() - depth as i32)
            .cloned()
            .collect();

        for board in boards_to_remove {
            self.table.remove(&board);
        }
    }
}
