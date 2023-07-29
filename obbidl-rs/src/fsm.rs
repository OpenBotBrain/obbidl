use std::{collections::HashSet, fmt, hash};

use crate::ast::Payload;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(u32);

#[derive(Debug, Clone)]
pub struct Transistion {
    pub start: State,
    pub end: State,
    pub label: String,
    pub payload: Payload,
}

#[derive(Debug, Clone)]
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
    pub fn graph_viz(&self) -> GraphViz {
        GraphViz(self)
    }
}

#[derive(Debug, Clone)]
pub struct GraphViz<'a>(&'a StateMachine);

impl<'a> fmt::Display for GraphViz<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "digraph {{")?;
        for trans in self.0.iter_transitions() {
            writeln!(
                f,
                "  {} -> {}[label=\"{}\"];",
                trans.start.0, trans.end.0, trans.label
            )?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

pub struct StateName(State);

impl State {
    pub fn name(&self) -> StateName {
        StateName(*self)
    }
}

impl fmt::Display for StateName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{}", self.0 .0)
    }
}
