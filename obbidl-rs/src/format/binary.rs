use std::fmt;

use crate::validate::{Message, Type};

use super::Format;

pub struct Binary;

impl Format for Binary {
    fn send_message(
        f: &mut fmt::Formatter<'_>,
        message: &Message,
        insert_tag: bool,
    ) -> fmt::Result {
        if insert_tag {
            writeln!(f, "self.0.send_u8({})?;", message.id)?;
        }

        for (name, ty) in &message.payload.items {
            send_type(f, name, ty)?;
        }

        writeln!(f, "return Ok({}(self.0));", message.dest_state_name)?;

        Ok(())
    }

    fn recv_messages(f: &mut fmt::Formatter<'_>, messages: &[Message]) -> fmt::Result {
        if messages.len() == 1 {
            recv_msg(f, &messages[0])?;
        } else {
            writeln!(f, "let id = self.0.recv_u8()?;")?;
            for msg in messages {
                writeln!(f, "if id == {} {{", msg.id)?;
                recv_msg(f, msg)?;
                writeln!(f, "}}")?;
            }
            writeln!(f, "panic!(\"invalid message!\")")?;
        }
        Ok(())
    }
}

fn recv_msg(f: &mut fmt::Formatter<'_>, msg: &Message) -> fmt::Result {
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
                Some(size) => writeln!(f, "let mut {} = [{}::default(); {}];", name, ty, size)?,
                None => writeln!(
                    f,
                    "let mut {} = vec![{}::default(); self.0.recv_u32()? as usize];",
                    name, ty
                )?,
            }
            writeln!(f, "for i in 0..{}.len() {{", name)?;
            recv_type(f, "x", ty)?;
            writeln!(f, "{}[i] = x;", name)?;
            writeln!(f, "}}")?;
        }
        Type::Struct(struct_) => {
            for (field_name, ty) in &struct_.fields {
                recv_type(f, &field_name, ty)?;
            }
            write!(f, "let {} = super::super::{} {{", name, struct_.name)?;
            for (field_name, _) in &struct_.fields {
                write!(f, "{},", field_name)?;
            }
            writeln!(f, "}};")?;
        }
    }
    Ok(())
}

fn send_type(f: &mut fmt::Formatter<'_>, name: &str, ty: &Type) -> fmt::Result {
    match ty {
        Type::Bool => writeln!(f, "self.0.send_u8(if {} {{ 1 }} else {{ 0 }})?;", name)?,
        Type::Int(ty) => writeln!(f, "self.0.send(&{}::to_be_bytes({}))?;", ty, name)?,
        Type::Array(ty, size) => {
            if size.is_none() {
                writeln!(f, "self.0.send(&u32::to_be_bytes({}.len() as u32))?;", name)?;
            }
            writeln!(f, "for i in 0..{}.len() {{", name)?;
            send_type(f, &format!("{}[i]", name), ty)?;
            writeln!(f, "}}")?;
        }
        Type::Struct(struct_) => {
            for (field_name, ty) in &struct_.fields {
                send_type(f, &format!("{}.{}", name, field_name), ty)?;
            }
        }
    }
    Ok(())
}
