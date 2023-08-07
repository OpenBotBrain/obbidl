use std::fmt;

use crate::ast::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(u32);

#[derive(Debug, Clone)]
pub struct Transition {
    pub start: State,
    pub end: State,
    pub msg: Message,
}

#[derive(Debug, Clone)]
pub struct StateMachine {
    state_count: u32,
    transitions: Vec<Transition>,
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.msg.label == other.msg.label
    }
}

impl Eq for Transition {}

impl StateMachine {
    pub fn new() -> StateMachine {
        StateMachine {
            state_count: 0,
            transitions: vec![],
        }
    }
    pub fn new_state(&mut self) -> State {
        self.state_count += 1;
        State(self.state_count - 1)
    }
    pub fn contains_state(&self, state: State) -> bool {
        state.0 < self.state_count
    }
    pub fn add_transition(&mut self, transition: Transition) {
        if !self.contains_state(transition.start) {
            panic!()
        }
        if !self.contains_state(transition.end) {
            panic!()
        }
        self.transitions.push(transition);
    }
    pub fn iter_transitions(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.iter()
    }
    pub fn iter_states(&self) -> impl Iterator<Item = State> + '_ {
        (0..self.state_count).map(|id| State(id))
    }
    pub fn iter_trans_from(&self, start: State) -> impl Iterator<Item = (&Message, State)> {
        self.iter_transitions().filter_map(move |trans| {
            if trans.start == start {
                Some((&trans.msg, trans.end))
            } else {
                None
            }
        })
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
                trans.start.0, trans.end.0, trans.msg.label,
            )?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
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
