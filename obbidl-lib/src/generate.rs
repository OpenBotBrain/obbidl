use std::{fmt, marker::PhantomData};

use crate::{
    ast::{IntSize, IntType},
    format::Format,
    validate::{Direction, File, Payload, Protocol, SimpleRole, Type},
};

impl fmt::Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.signed {
            write!(f, "i")?;
        } else {
            write!(f, "u")?;
        }
        match self.size {
            IntSize::B64 => write!(f, "64"),
            IntSize::B32 => write!(f, "32"),
            IntSize::B16 => write!(f, "16"),
            IntSize::B8 => write!(f, "8"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Int(ty) => write!(f, "{}", ty),
            Type::Array(ty, size) => match size {
                Some(size) => write!(f, "[{}; {}]", ty, size),
                None => write!(f, "Vec<{}>", ty),
            },
            Type::Struct(struct_) => write!(f, "super::super::{}", struct_.name),
        }
    }
}

struct BorrowedType<'a>(&'a Type);

impl<'a> fmt::Display for BorrowedType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Type::Bool => write!(f, "bool"),
            Type::Int(ty) => write!(f, "{}", ty),
            Type::Array(ty, size) => match size {
                Some(size) => write!(f, "&[{}; {}]", ty, size),
                None => write!(f, "&[{}]", ty),
            },
            Type::Struct(struct_) => write!(f, "&super::super::{}", struct_.name),
        }
    }
}

struct BorrowedPayload<'a>(&'a Payload);

impl<'a> fmt::Display for BorrowedPayload<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.0.items {
            write!(f, "{}: {}, ", name, BorrowedType(ty))?;
        }
        Ok(())
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.items {
            write!(f, "{}: {}, ", name, ty)?;
        }
        Ok(())
    }
}

fn generate_protocol<F: Format>(
    f: &mut fmt::Formatter<'_>,
    protocol: &Protocol,
    role: SimpleRole,
) -> fmt::Result {
    writeln!(f, "use std::mem::size_of;")?;
    writeln!(f, "use obbidl_lib::channel::Channel;")?;

    for state in &protocol.states {
        writeln!(f, "#[must_use]")?;
        writeln!(f, "pub struct {}<C: Channel>(C);", state.name)?;

        if let Some(trans) = &state.trans {
            if (trans.dir == Direction::AToB && role == SimpleRole::B)
                || (trans.dir == Direction::BToA && role == SimpleRole::A)
            {
                writeln!(
                    f,
                    "pub trait {}Receiver<C: Channel<Error = E>, E> {{",
                    state.name
                )?;
                writeln!(f, "type Type;")?;

                for msg in &trans.messages {
                    writeln!(
                        f,
                        "fn recv_{}(self, state: {}<C>, {}) -> Result<Self::Type, E>;",
                        msg.label, msg.dest_state_name, msg.payload
                    )?;
                }
                writeln!(f, "}}")?;

                writeln!(f, "pub enum {}Response<C: Channel> {{", state.name)?;
                for msg in &trans.messages {
                    writeln!(f, "#[allow(non_camel_case_types)]")?;
                    writeln!(f, "{} {{", msg.label)?;
                    writeln!(f, "state: {}<C>, {}", msg.dest_state_name, msg.payload)?;
                    writeln!(f, "}},")?;
                }
                writeln!(f, "}}")?;

                writeln!(f, "struct {}DefaultReceiver;", state.name)?;

                writeln!(
                    f,
                    "impl<C: Channel<Error = E>, E> {}Receiver<C, E> for {}DefaultReceiver {{",
                    state.name, state.name
                )?;
                writeln!(f, "type Type = {}Response<C>;", state.name)?;
                for msg in &trans.messages {
                    writeln!(
                        f,
                        "fn recv_{}(self, state: {}<C>, {}) -> Result<Self::Type, E> {{",
                        msg.label, msg.dest_state_name, msg.payload
                    )?;
                    write!(f, "Ok({}Response::{} {{ state, ", state.name, msg.label)?;
                    for (name, _) in &msg.payload.items {
                        write!(f, "{}, ", name)?;
                    }
                    writeln!(f, "}})")?;
                    writeln!(f, "}}")?;
                }
                writeln!(f, "}}")?;

                writeln!(f, "impl<C: Channel<Error = E>, E> {}<C> {{", state.name)?;
                writeln!(f, "pub fn recv<T>(mut self, receiver: impl {}Receiver<C, E, Type = T>) -> Result<T, E> {{", state.name)?;
                F::recv_messages(f, &trans.messages)?;
                writeln!(f, "}}")?;

                writeln!(
                    f,
                    "pub fn recv_default(self) -> Result<{}Response<C>, E> {{",
                    state.name
                )?;
                writeln!(f, "self.recv({}DefaultReceiver)", state.name)?;
                writeln!(f, "}}")?;
            }

            if (trans.dir == Direction::AToB && role == SimpleRole::A)
                || (trans.dir == Direction::BToA && role == SimpleRole::B)
            {
                writeln!(f, "impl<C: Channel<Error = E>, E> {}<C> {{", state.name)?;

                for msg in &trans.messages {
                    writeln!(
                        f,
                        "pub fn send_{}(mut self, {}) -> Result<{}<C>, E> {{",
                        msg.label,
                        BorrowedPayload(&msg.payload),
                        msg.dest_state_name
                    )?;

                    F::send_message(f, msg, trans.messages.len() > 1)?;

                    writeln!(f, "}}")?;
                }
            }
            writeln!(f, "}}")?;
        } else {
            writeln!(f, "impl<C: Channel<Error = E>, E> {}<C> {{", state.name)?;
            writeln!(f, "pub fn finish(self) {{}}")?;
            writeln!(f, "}}")?;
        }
    }

    writeln!(f, "impl<C: Channel> S0<C> {{")?;
    writeln!(f, "pub fn new(channel: C) -> S0<C> {{")?;
    writeln!(f, "S0(channel)")?;
    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    Ok(())
}

fn generate_protocol_file<F: Format>(f: &mut fmt::Formatter<'_>, file: &File) -> fmt::Result {
    for struct_ in &file.structs {
        writeln!(f, "pub struct {} {{", struct_.name)?;
        for (name, ty) in &struct_.fields {
            writeln!(f, "{}: {},", name, ty)?;
        }
        writeln!(f, "}}")?;
    }

    for protocol in &file.protocols {
        writeln!(f, "pub mod {} {{", protocol.name)?;

        writeln!(f, "pub mod {} {{", protocol.role_a)?;
        generate_protocol::<F>(f, protocol, SimpleRole::A)?;
        writeln!(f, "}}")?;

        writeln!(f, "pub mod {} {{", protocol.role_b)?;
        generate_protocol::<F>(f, protocol, SimpleRole::B)?;
        writeln!(f, "}}")?;

        writeln!(f, "}}")?;
    }

    Ok(())
}

pub struct GenerateRust<'a, F: Format>(&'a File, PhantomData<F>);

impl<'a, F: Format> GenerateRust<'a, F> {
    pub fn new(file: &'a File) -> GenerateRust<'a, F> {
        GenerateRust(file, PhantomData)
    }
}

impl<'a, F: Format> fmt::Display for GenerateRust<'a, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        generate_protocol_file::<F>(f, self.0)
    }
}
