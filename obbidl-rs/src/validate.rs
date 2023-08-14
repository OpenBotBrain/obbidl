use std::{collections::HashSet, fmt};

use askama::Template;

use crate::{
    ast::{IntSize, Role, Type},
    compile::{ProtocolFileStateMachines, ProtocolStateMachine},
    state_machine::StateName,
};

// #[derive(Debug, Clone, Template)]
// #[template(path = "rust.jinja", whitespace = "minimize")]
// struct RustTemplate {
//     file: ProtocolFile,
// }

#[derive(Debug, Clone)]
pub struct ProtocolFile {
    pub protocols: Vec<Protocol>,
}

#[derive(Debug, Clone)]
pub struct Protocol {
    pub name: String,
    pub role_a: Role,
    pub role_b: Role,
    pub states: Vec<State>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub name: StateName,
    pub trans: Option<Transitions>,
}

#[derive(Debug, Clone)]
pub struct Transitions {
    pub dir: Direction,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    BToA,
    AToB,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimpleRole {
    A,
    B,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub label: String,
    pub id: u8,
    pub payload: Payload,
    pub dest_state_name: StateName,
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub items: Vec<(String, Type)>,
}

// pub fn generate_rust_bindings(file: &ProtocolFileStateMachines) -> Result<String, ErrorInfo> {
//     RustTemplate {
//         file: validate_protocol_file(file),
//     }
//     .render()
//     .unwrap()
// }

pub fn validate_protocol_file(file: &ProtocolFileStateMachines) -> Result<ProtocolFile, ErrorInfo> {
    let mut protocols = vec![];
    for protocol in &file.protocols {
        protocols.push(validate_protocol(protocol).map_err(|err| ErrorInfo {
            name: protocol.name.clone(),
            err,
        })?);
    }

    Ok(ProtocolFile { protocols })
}

pub struct ErrorInfo {
    name: String,
    err: Error,
}

pub enum Error {
    IncorrectNumberOfRoles,
    InvalidDirection,
    MixedDirections,
    RepeatedLabel,
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "validation error in protocol {}:", self.name)?;
        match self.err {
            Error::IncorrectNumberOfRoles => write!(f, "there must be two roles in the protocol"),
            Error::InvalidDirection => write!(f, "there is a message sending something from and to the same role"),
            Error::MixedDirections => write!(f, "there is a decision point in the protocol where there are a mix of directions"),
            Error::RepeatedLabel => write!(f, "there is a decision point in the protocol where the same label is used more than once"),
        }
    }
}

pub fn validate_protocol(protocol: &ProtocolStateMachine) -> Result<Protocol, Error> {
    let mut states = vec![];

    if protocol.roles.len() != 2 {
        return Err(Error::IncorrectNumberOfRoles);
    }

    let a = protocol.roles[0].clone();
    let b = protocol.roles[1].clone();

    for state in protocol.state_machine.iter_states() {
        let mut overall_dir = None;
        let mut labels = HashSet::new();
        let mut messages = vec![];

        for (index, (msg, final_state)) in protocol.state_machine.iter_trans_from(state).enumerate()
        {
            let dir = if msg.from == a && msg.to == b {
                Direction::AToB
            } else if msg.from == b && msg.to == a {
                Direction::BToA
            } else {
                return Err(Error::InvalidDirection);
            };
            match overall_dir {
                Some(overall_dir) => {
                    if overall_dir != dir {
                        return Err(Error::MixedDirections);
                    }
                }
                None => overall_dir = Some(dir),
            }

            if !labels.insert(msg.label.as_str()) {
                return Err(Error::RepeatedLabel);
            }

            let items = msg
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
                .collect();

            messages.push(Message {
                label: msg.label.clone(),
                id: index as u8,
                payload: Payload { items },
                dest_state_name: final_state.name(),
            })
        }

        states.push(State {
            name: state.name(),
            trans: match overall_dir {
                Some(dir) => Some(Transitions { dir, messages }),
                None => None,
            },
        })
    }

    Ok(Protocol {
        name: protocol.name.clone(),
        role_a: a,
        role_b: b,
        states,
    })
}
