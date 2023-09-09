use std::{collections::HashSet, fmt, rc::Rc};

use colored::Colorize;

use crate::{
    ast,
    compile::{ProtocolFileStateMachines, ProtocolStateMachine},
    parser::Span,
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
    pub role_a: ast::Role,
    pub role_b: ast::Role,
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
    Int(ast::IntType),
    Array(Box<Type>, Option<u64>),
    Struct(Rc<Struct>),
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

pub fn validate_protocol_file<'a>(
    file: &'a ProtocolFileStateMachines,
    structs: &'a [Span<ast::Struct>],
) -> Result<File, Vec<Error<'a>>> {
    let mut errors = vec![];

    let mut output_structs = vec![];
    for struct_ in structs {
        match validate_struct(
            &struct_.inner.name,
            structs,
            &mut HashSet::new(),
            &mut output_structs,
        ) {
            Ok(_) => (),
            Err(err) => errors.push(Error::StructError { struct_, err }),
        }
    }

    let mut protocols = vec![];
    for protocol in &file.protocols {
        match validate_protocol(&protocol.inner, &output_structs) {
            Ok(protocol) => protocols.push(protocol),
            Err(err) => errors.push(Error::ProtocolError { protocol, err }),
        }
    }

    if errors.len() > 0 {
        return Err(errors);
    }

    Ok(File {
        protocols,
        structs: output_structs,
    })
}

pub fn validate_struct<'a>(
    name: &'a str,
    structs: &'a [Span<ast::Struct>],
    previous_structs: &mut HashSet<&'a str>,
    output_structs: &mut Vec<Rc<Struct>>,
) -> Result<Rc<Struct>, StructError<'a>> {
    if !previous_structs.insert(name) {
        return Err(StructError::RecursiveStruct(name));
    }
    let fields = structs
        .iter()
        .find(|struct_| &struct_.inner.name == name)
        .ok_or(StructError::UndefinedStruct(name))?
        .inner
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
    structs: &'a [Span<ast::Struct>],
    previous_structs: &mut HashSet<&'a str>,
    output_structs: &mut Vec<Rc<Struct>>,
) -> Result<Type, StructError<'a>> {
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

pub fn validate_type_ref<'a>(
    ty: &'a ast::Type,
    structs: &[Rc<Struct>],
) -> Result<Type, ProtocolError<'a>> {
    Ok(match ty {
        ast::Type::Bool => Type::Bool,
        ast::Type::Int(ty) => Type::Int(*ty),
        ast::Type::Array(ty, size) => {
            Type::Array(Box::new(validate_type_ref(&ty, structs)?), *size)
        }
        ast::Type::Struct(name) => Type::Struct(Rc::clone(
            structs
                .iter()
                .find(|struct_| &*struct_.name == name)
                .ok_or(ProtocolError::UndefinedStruct(name))?,
        )),
    })
}

#[derive(Debug, Clone)]
pub enum Error<'a> {
    ProtocolError {
        protocol: &'a Span<ProtocolStateMachine>,
        err: ProtocolError<'a>,
    },
    StructError {
        struct_: &'a Span<ast::Struct>,
        err: StructError<'a>,
    },
}

pub struct PrettyPrintError<'a> {
    error: &'a Error<'a>,
    source: &'a str,
}

#[derive(Debug, Clone)]
pub enum ProtocolError<'a> {
    IncorrectNumberOfRoles,
    InvalidDirection(&'a Span<ast::Message>),
    MixedDirections(Vec<&'a Span<ast::Message>>),
    RepeatedLabel(Vec<&'a Span<ast::Message>>),
    UndefinedStruct(&'a str),
}

#[derive(Debug, Clone)]
pub enum StructError<'a> {
    UndefinedStruct(&'a str),
    RecursiveStruct(&'a str),
}

impl<'a> Error<'a> {
    pub fn pretty_print(&'a self, source: &'a str) -> PrettyPrintError<'a> {
        PrettyPrintError {
            error: self,
            source,
        }
    }
}

impl<'a> fmt::Display for PrettyPrintError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", "validation error".red())?;
        match self.error {
            Error::ProtocolError { protocol, err } => {
                writeln!(f, "error in protocol '{}'", &protocol.inner.name)?;
                write!(f, "{}", protocol.pretty_print(self.source))?;

                match err {
                    ProtocolError::IncorrectNumberOfRoles => {
                        writeln!(
                            f,
                            "info: the protocol defined above has {} role(s) but it is required to have 2",
                            protocol.inner.roles.len()
                        )?;
                    }
                    ProtocolError::InvalidDirection(msg) => {
                        writeln!(
                            f,
                            "info: the following message is to '{}' and from '{}'",
                            msg.inner.to, msg.inner.from
                        )?;
                        write!(f, "{}", msg.pretty_print(self.source))?;
                        writeln!(
                            f,
                            "info: modify the messages so the sender and receiver are not the same role"
                        )?;
                    }
                    ProtocolError::MixedDirections(messages) => {
                        writeln!(f, "info: the following messages are part of the same decision state but have different directions:")?;
                        for msg in messages {
                            write!(f, "{}", msg.pretty_print(self.source))?;
                        }
                        writeln!(
                            f,
                            "info: make sure all the messages have the same roles in the 'from' and 'to' section"
                        )?;
                    }
                    ProtocolError::RepeatedLabel(messages) => {
                        writeln!(f, "info: the following messages are part of the same decision state but have the same label:")?;
                        for msg in messages {
                            write!(f, "{}", msg.pretty_print(self.source))?;
                        }
                        writeln!(f, "info: rename the message labels so they are unique")?;
                    }
                    ProtocolError::UndefinedStruct(name) => {
                        writeln!(
                            f,
                            "info: the struct '{}' is not defined anywhere in the file",
                            name
                        )?;
                        writeln!(
                            f,
                            "info: either define this struct or change the type to a struct that exists"
                        )?;
                    }
                }
            }
            Error::StructError { struct_, err } => {
                writeln!(f, "error in struct definition '{}'", &struct_.inner.name)?;
                write!(f, "{}", struct_.pretty_print(self.source))?;
                match err {
                    StructError::RecursiveStruct(_) => {
                        writeln!(f, "info: the struct below contains a recursive definition")?;
                        writeln!(f, "info: remove the recursive definition")?;
                    }
                    StructError::UndefinedStruct(name) => {
                        writeln!(
                            f,
                            "info: the struct '{}' is not defined anywhere in the file",
                            name
                        )?;
                        writeln!(
                            f,
                            "info: either define this struct or change the type to a struct that exists"
                        )?;
                    }
                }
            }
        };
        Ok(())
    }
}

pub fn validate_protocol<'a>(
    protocol: &'a ProtocolStateMachine,
    structs: &[Rc<Struct>],
) -> Result<Protocol, ProtocolError<'a>> {
    let mut states = vec![];

    if protocol.roles.len() != 2 {
        return Err(ProtocolError::IncorrectNumberOfRoles);
    }

    let a = protocol.roles[0].clone();
    let b = protocol.roles[1].clone();

    for state in protocol.state_machine.iter_states() {
        let mut overall_dir = None;
        let mut labels = HashSet::new();
        let mut messages = vec![];

        for (index, (msg, final_state)) in protocol.state_machine.iter_trans_from(state).enumerate()
        {
            let dir = if msg.inner.from == a && msg.inner.to == b {
                Direction::AToB
            } else if msg.inner.from == b && msg.inner.to == a {
                Direction::BToA
            } else {
                return Err(ProtocolError::InvalidDirection(msg));
            };
            match overall_dir {
                Some(overall_dir) => {
                    if overall_dir != dir {
                        return Err(ProtocolError::MixedDirections(
                            protocol
                                .state_machine
                                .iter_trans_from(state)
                                .map(|(msg, _)| msg)
                                .collect(),
                        ));
                    }
                }
                None => overall_dir = Some(dir),
            }

            if !labels.insert(msg.inner.label.as_str()) {
                let msgs = protocol
                    .state_machine
                    .iter_trans_from(state)
                    .filter_map(|(m, _)| {
                        if m.inner.label == msg.inner.label {
                            Some(m)
                        } else {
                            None
                        }
                    })
                    .collect();
                return Err(ProtocolError::RepeatedLabel(msgs));
            }

            let items = msg
                .inner
                .payload
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Ok((
                        item.name
                            .clone()
                            .unwrap_or_else(|| format!("param{}", index)),
                        validate_type_ref(&item.ty, structs)?,
                    ))
                })
                .collect::<Result<_, _>>()?;

            messages.push(Message {
                label: msg.inner.label.clone(),
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
