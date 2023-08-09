use askama::Template;

use crate::{
    ast::{IntSize, Role, Type},
    compile::{ProtocolFileStateMachines, ProtocolStateMachine},
    state_machine::StateName,
};

#[derive(Debug, Clone, Template)]
#[template(path = "rust.jinja")]
struct RustTemplate {
    file: ProtocolFile,
}

#[derive(Debug, Clone)]
pub struct ProtocolFile {
    protocols: Vec<Protocol>,
}

#[derive(Debug, Clone)]
pub struct Protocol {
    name: String,
    role_a: Role,
    role_b: Role,
    states: Vec<State>,
}

#[derive(Debug, Clone)]
struct State {
    name: StateName,
    trans: Option<Transitions>,
}

#[derive(Debug, Clone)]
struct Transitions {
    dir: Direction,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    BToA,
    AToB,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SimpleRole {
    A,
    B,
}

#[derive(Debug, Clone)]
struct Message {
    label: String,
    id: u8,
    payload: Payload,
    dest_state_name: StateName,
}

#[derive(Debug, Clone)]
struct Payload {
    items: Vec<(String, Type)>,
}

pub fn generate_rust_bindings(file: &ProtocolFileStateMachines) -> String {
    RustTemplate {
        file: validate_protocol_file(file),
    }
    .render()
    .unwrap()
}

pub fn validate_protocol_file(file: &ProtocolFileStateMachines) -> ProtocolFile {
    let mut protocols = vec![];
    for protocol in &file.protocols {
        protocols.push(validate_protocol(protocol));
    }

    ProtocolFile { protocols }
}

pub fn validate_protocol(protocol: &ProtocolStateMachine) -> Protocol {
    let mut states = vec![];

    if protocol.roles.len() != 2 {
        panic!()
    }

    let a = protocol.roles[0].clone();
    let b = protocol.roles[1].clone();

    for state in protocol.state_machine.iter_states() {
        let mut dir_iter = protocol
            .state_machine
            .iter_trans_from(state)
            .map(|(msg, _)| {
                if msg.from == a && msg.to == b {
                    Direction::AToB
                } else if msg.from == b && msg.to == a {
                    Direction::BToA
                } else {
                    panic!()
                }
            });

        let trans = if let Some(dir) = dir_iter.next() {
            if !dir_iter.all(|d| d == dir) {
                panic!()
            }

            let messages = protocol
                .state_machine
                .iter_trans_from(state)
                .enumerate()
                .map(|(index, (msg, state))| Message {
                    label: msg.label.clone(),
                    id: index as u8,
                    payload: Payload {
                        items: msg
                            .payload
                            .items
                            .iter()
                            .enumerate()
                            .map(|(index, (name, ty))| {
                                (
                                    name.clone().unwrap_or_else(|| format!("param{}", index)),
                                    ty.clone(),
                                )
                            })
                            .collect(),
                    },
                    dest_state_name: state.name(),
                })
                .collect();

            // CHECK FOR MULTIPLE OF SAME LABEL!!!

            Some(Transitions { dir, messages })
        } else {
            None
        };

        states.push(State {
            name: state.name(),
            trans,
        })
    }

    Protocol {
        name: protocol.name.clone(),
        role_a: a,
        role_b: b,
        states,
    }
}
