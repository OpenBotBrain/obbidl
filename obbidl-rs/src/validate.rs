use std::{collections::HashSet, fmt, rc::Rc};

use crate::{
    ast::{self, IntType, Role},
    compile::{ProtocolFileStateMachines, ProtocolStateMachine},
    state_machine::StateName,
};

#[derive(Debug, Clone)]
pub struct File {
    pub protocols: Vec<Protocol>,
    pub structs: Vec<Rc<Struct>>,
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

#[derive(Debug, Clone)]
pub enum Type {
    Bool,
    Int(IntType),
    Array(Box<Type>, Option<u64>),
    Struct(Rc<Struct>),
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

pub fn validate_protocol_file(
    file: &ProtocolFileStateMachines,
    structs: &[ast::Struct],
) -> Result<File, ErrorInfo> {
    let mut output_structs = vec![];
    for struct_ in structs {
        validate_struct(
            &struct_.name,
            structs,
            &mut HashSet::new(),
            &mut output_structs,
        )
        .unwrap();
    }

    let mut protocols = vec![];
    for protocol in &file.protocols {
        protocols.push(
            validate_protocol(protocol, &output_structs).map_err(|err| ErrorInfo {
                name: protocol.name.clone(),
                err,
            })?,
        );
    }

    Ok(File {
        protocols,
        structs: output_structs,
    })
}

pub fn validate_struct<'a>(
    name: &'a str,
    structs: &'a [ast::Struct],
    previous_structs: &mut HashSet<&'a str>,
    output_structs: &mut Vec<Rc<Struct>>,
) -> Result<Rc<Struct>, Error> {
    if !previous_structs.insert(name) {
        return Err(Error::RecursiveStruct);
    }
    let fields = structs
        .iter()
        .find(|struct_| &struct_.name == name)
        .ok_or(Error::UndefinedStruct)?
        .fields
        .iter()
        .map(|(name, ty)| {
            Ok((
                name.clone(),
                validate_type(ty, structs, previous_structs, output_structs)?,
            ))
        })
        .collect::<Result<_, _>>()?;

    let struct_ = Rc::new(Struct {
        name: name.to_string(),
        fields,
    });

    output_structs.push(Rc::clone(&struct_));

    Ok(struct_)
}

pub fn validate_type<'a>(
    ty: &'a ast::Type,
    structs: &'a [ast::Struct],
    previous_structs: &mut HashSet<&'a str>,
    output_structs: &mut Vec<Rc<Struct>>,
) -> Result<Type, Error> {
    Ok(match ty {
        ast::Type::Bool => Type::Bool,
        ast::Type::Int(ty) => Type::Int(*ty),
        ast::Type::Array(ty, size) => Type::Array(
            Box::new(validate_type(
                &ty,
                structs,
                previous_structs,
                output_structs,
            )?),
            *size,
        ),
        ast::Type::Struct(name) => Type::Struct(validate_struct(
            name,
            structs,
            previous_structs,
            output_structs,
        )?),
    })
}

pub fn validate_type_simple(ty: &ast::Type, structs: &[Rc<Struct>]) -> Result<Type, Error> {
    Ok(match ty {
        ast::Type::Bool => Type::Bool,
        ast::Type::Int(ty) => Type::Int(*ty),
        ast::Type::Array(ty, size) => {
            Type::Array(Box::new(validate_type_simple(&ty, structs)?), *size)
        }
        ast::Type::Struct(name) => Type::Struct(Rc::clone(
            structs
                .iter()
                .find(|struct_| &*struct_.name == name)
                .ok_or(Error::UndefinedStruct)?,
        )),
    })
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    name: String,
    err: Error,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    IncorrectNumberOfRoles,
    InvalidDirection,
    MixedDirections,
    RepeatedLabel,
    UndefinedStruct,
    RecursiveStruct,
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "validation error in protocol {}:", self.name)?;
        match self.err {
            Error::IncorrectNumberOfRoles => write!(f, "there must be two roles in the protocol"),
            Error::InvalidDirection => write!(f, "there is a message sending something from and to the same role"),
            Error::MixedDirections => write!(f, "there is a decision point in the protocol where there are a mix of directions"),
            Error::RepeatedLabel => write!(f, "there is a decision point in the protocol where the same label is used more than once"),
            Error::UndefinedStruct => write!(f, "a struct is used but is not defined anywhere in the file"),
            Error::RecursiveStruct => write!(f, "recursive struct definition"),
        }
    }
}

pub fn validate_protocol(
    protocol: &ProtocolStateMachine,
    structs: &[Rc<Struct>],
) -> Result<Protocol, Error> {
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
                    Ok((
                        name.clone().unwrap_or_else(|| format!("param{}", index)),
                        validate_type_simple(ty, structs)?,
                    ))
                })
                .collect::<Result<_, _>>()?;

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
