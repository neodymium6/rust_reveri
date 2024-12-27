use crate::board::core::Board;
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
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn store(&mut self, board: &Board, depth: usize, score: i32, entry_type: EntryType) {
        if let Some(entry) = self.table.get(board) {
            if entry.get_depth() > depth {
                return;
            }
            if entry.get_type() == EntryType::Exact && entry.get_depth() == depth {
                return;
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
}
