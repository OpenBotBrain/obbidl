use std::fmt;

use crate::{
    ast::{IntSize, IntType, Type},
    generate::{Direction, Payload, Protocol, ProtocolFile, SimpleRole},
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
                Some(size) => write!(f, "&[{}; {}]", ty, size),
                None => write!(f, "&[{}]", ty),
            },
        }
    }
}

fn send_type(f: &mut fmt::Formatter<'_>, name: &str, ty: &Type) -> fmt::Result {
    match ty {
        Type::Bool => writeln!(f, "self.0.send_u8(if {} {{ 1 }} else {{ 0 }})?;", name)?,
        Type::Int(ty) => writeln!(f, "self.0.send(&{}::to_be_bytes({}))?;", ty, name)?,
        Type::Array(ty, size) => {
            if size.is_none() {
                writeln!(f, "self.send(&u32::to_be_bytes({}.len()));", name)?;
            }
            writeln!(f, "for i in 0..{}.len() {{", name)?;
            send_type(f, &format!("{}[i]", name), ty)?;
            writeln!(f, "}}")?;
        }
    }
    Ok(())
}

fn recv_type(f: &mut fmt::Formatter<'_>, name: &str, ty: &Type) -> fmt::Result {
    match ty {
        Type::Bool => writeln!(f, "let {} = self.0.recv_u8()? != 0;", name)?,
        Type::Int(ty) => {
            writeln!(f, "let mut bytes = [0; size_of::<{}>()];", ty)?;
            writeln!(f, "self.0.recv(&mut bytes)?;")?;
            writeln!(f, "let {} = {}::from_be_bytes(bytes);", name, ty)?;
        }
        Type::Array(ty, size) => {
            match size {
                Some(size) => writeln!(f, "let {} = [{}::default(); {}];", name, ty, size)?,
                None => writeln!(
                    f,
                    "let {} = vec![{}::default(); self.0.recv_u32()];",
                    name, ty
                )?,
            }
            writeln!(f, "for i in 0..{}.len() {{", name)?;
            recv_type(f, "x", ty)?;
            writeln!(f, "{}[i] = x;", name)?;
            writeln!(f, "}}")?;
        }
    }
    Ok(())
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, ty) in &self.items {
            write!(f, "{}: {}, ", name, ty)?;
        }
        Ok(())
    }
}

fn generate_protocol(
    f: &mut fmt::Formatter<'_>,
    protocol: &Protocol,
    role: SimpleRole,
) -> fmt::Result {
    writeln!(f, "use std::mem::size_of;")?;
    writeln!(f, "use obbidl::channel::Channel;")?;

    for state in &protocol.states {
        writeln!(f, "#[must_use]")?;
        writeln!(f, "pub struct {}<C: Channel>(C);", state.name)?;

        match &state.trans {
            Some(trans) => {
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

                    writeln!(f, "impl<C: Channel<Error = E>, E> {}<C> {{", state.name)?;
                    writeln!(f, "pub fn recv<T>(mut self, receiver: impl {}Receiver<C, E, Type = T>) -> Result<T, E> {{", state.name)?;
                    writeln!(f, "let id = self.0.recv_u8()?;")?;
                    for msg in &trans.messages {
                        writeln!(f, "if id == {} {{", msg.id)?;
                        for (name, ty) in &msg.payload.items {
                            recv_type(f, name, ty)?;
                        }

                        write!(
                            f,
                            "return Ok(receiver.recv_{}({}(self.0), ",
                            msg.label, msg.dest_state_name
                        )?;
                        for (name, _) in &msg.payload.items {
                            write!(f, "{}, ", name)?;
                        }
                        writeln!(f, ")?);")?;
                        writeln!(f, "}}")?;
                    }
                    writeln!(f, "panic!(\"invalid message!\")")?;
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
                            msg.label, msg.payload, msg.dest_state_name
                        )?;
                        writeln!(f, "self.0.send_u8({})?;", msg.id)?;

                        for (name, ty) in &msg.payload.items {
                            send_type(f, name, ty)?;
                        }

                        writeln!(f, "Ok({}(self.0))", msg.dest_state_name)?;

                        writeln!(f, "}}")?;
                    }
                }
                writeln!(f, "}}")?;
            }
            None => {
                writeln!(f, "impl<C: Channel<Error = E>, E> {}<C> {{", state.name)?;
                writeln!(f, "pub fn finish(self) {{}}")?;
                writeln!(f, "}}")?;
            }
        }
    }

    writeln!(f, "impl<C: Channel> S0<C> {{")?;
    writeln!(f, "pub fn new(channel: C) -> S0<C> {{")?;
    writeln!(f, "S0(channel)")?;
    writeln!(f, "}}")?;
    writeln!(f, "}}")?;

    Ok(())
}

fn generate_protocol_file(f: &mut fmt::Formatter<'_>, file: &ProtocolFile) -> fmt::Result {
    for protocol in &file.protocols {
        writeln!(f, "pub mod {} {{", protocol.name)?;

        writeln!(f, "pub mod {} {{", protocol.role_a)?;
        generate_protocol(f, protocol, SimpleRole::A)?;
        writeln!(f, "}}")?;

        writeln!(f, "pub mod {} {{", protocol.role_b)?;
        generate_protocol(f, protocol, SimpleRole::B)?;
        writeln!(f, "}}")?;

        writeln!(f, "}}")?;
    }

    Ok(())
}

pub struct GenerateRust<'a>(pub &'a ProtocolFile);

impl<'a> fmt::Display for GenerateRust<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        generate_protocol_file(f, self.0)
    }
}
