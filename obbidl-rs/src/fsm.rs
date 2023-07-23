use std::{collections::HashSet, hash};

use crate::ast::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(u32);

#[derive(Debug, Clone)]
pub struct Transistion {
    pub start: State,
    pub end: State,
    pub label: String,
    pub payload: Option<Vec<(String, Type)>>,
}

#[derive(Debug)]
pub struct StateMachine {
    state_count: u32,
    transitions: HashSet<Transistion>,
}

impl PartialEq for Transistion {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.label == other.label
    }
}

impl hash::Hash for Transistion {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.label.hash(state);
    }
}

impl Eq for Transistion {}

impl StateMachine {
    pub fn new() -> StateMachine {
        StateMachine {
            state_count: 0,
            transitions: HashSet::new(),
        }
    }
    pub fn new_state(&mut self) -> State {
        self.state_count += 1;
        State(self.state_count - 1)
    }
    pub fn contains_state(&self, state: State) -> bool {
        state.0 < self.state_count
    }
    pub fn add_transition(&mut self, transistion: Transistion) {
        if !self.contains_state(transistion.start) {
            panic!()
        }
        if !self.contains_state(transistion.end) {
            panic!()
        }
        self.transitions.insert(transistion);
    }
    pub fn iter_transitions(&self) -> impl Iterator<Item = &Transistion> {
        self.transitions.iter()
    }
    pub fn iter_states(&self) -> impl Iterator<Item = State> + '_ {
        (0..self.state_count).map(|id| State(id))
    }
}
