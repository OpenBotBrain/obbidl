use std::fmt;

use crate::{ast::Message, compile::ProtocolFileStateMachines, graph::GraphViz, parser::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(pub u32);

#[derive(Debug, Clone)]
pub struct Transition {
    pub start: State,
    pub end: State,
    pub msg: Span<Message>,
}

#[derive(Debug, Clone)]
pub struct StateMachine {
    state_count: u32,
    transitions: Vec<Transition>,
}

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
    pub fn iter_trans_from(&self, start: State) -> impl Iterator<Item = (&Span<Message>, State)> {
        self.iter_transitions().filter_map(move |trans| {
            if trans.start == start {
                Some((&trans.msg, trans.end))
            } else {
                None
            }
        })
    }
}

impl ProtocolFileStateMachines {
    pub fn graph_viz(&self) -> GraphViz {
        GraphViz(self)
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
