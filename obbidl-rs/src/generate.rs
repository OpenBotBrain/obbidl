use askama::Template;

use crate::{
    ast::{Payload, Role, Type},
    fsm::StateName,
    to_fsm::ProtocolStateMachine,
};

#[derive(Debug, Clone, Template)]
#[template(path = "rust.jinja")]
struct RustTemplate {
    states: Vec<State>,
}

#[derive(Debug, Clone)]
struct State {
    name: StateName,
    dir: Direction,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Send,
    Recv,
}

#[derive(Debug, Clone)]
struct Message {
    label: String,
    payload: Payload,
    dest_state_name: StateName,
}

pub fn generate_rust_bindings(protocol: &ProtocolStateMachine, you: Role, other: Role) -> String {
    let mut states = vec![];

    for state in protocol.state_machine.iter_states() {
        let mut dir_iter = protocol
            .state_machine
            .iter_trans_from(state)
            .map(|(msg, _)| {
                if msg.from == you && msg.to == other {
                    Direction::Send
                } else if msg.from == other && msg.to == you {
                    Direction::Recv
                } else {
                    panic!()
                }
            });

        if let Some(dir) = dir_iter.next() {
            if !dir_iter.all(|d| d == dir) {
                panic!()
            }

            let messages = protocol
                .state_machine
                .iter_trans_from(state)
                .map(|(msg, state)| Message {
                    label: msg.label.clone(),
                    payload: msg.payload.clone(),
                    dest_state_name: state.name(),
                })
                .collect();

            states.push(State {
                name: state.name(),
                dir,
                messages,
            })
        }
    }

    RustTemplate { states }.render().unwrap()
}
