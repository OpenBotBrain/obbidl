use std::fmt;

use crate::validate::{Message, Type};

use super::Format;

struct Json;

impl Format for Json {
    fn send_message(
        f: &mut fmt::Formatter<'_>,
        message: &Message,
        insert_tag: bool,
    ) -> fmt::Result {
        writeln!(f, "let mut object = HashMap::new()")?;
        if insert_tag {
            writeln!(f, "object.insert(\"label\", \"{}\".into());", message.label)?;
        }
        for (name, ty) in &message.payload.items {
            to_json_value(f, name, ty)?;
            writeln!(f, "object.insert(\"{}\", value);", name)?;
        }

        // send json value

        Ok(())
    }

    fn recv_messages(f: &mut fmt::Formatter<'_>, messages: &[Message]) -> fmt::Result {
        writeln!(f, "let value = ")?;
        if messages.len() == 1 {
            msg_to_json(f, &messages[0])?;
        } else {
            writeln!(f, "let label = value[\"label\"].as_str().unwrap();")?;
            for msg in messages {
                writeln!(f, "if label == \"{}\" {{", msg.label)?;
                msg_to_json(f, msg)?;
                writeln!(f, "}}")?;
            }
            writeln!(f, "panic!(\"invalid message!\")")?;
        }
        Ok(())
    }
}

fn msg_to_json(f: &mut fmt::Formatter<'_>, msg: &Message) -> fmt::Result {
    for (name, ty) in &msg.payload.items {
        from_json_value(f, name, ty)?;
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

fn from_json_value(f: &mut fmt::Formatter<'_>, name: &str, ty: &Type) -> fmt::Result {
    match ty {
        Type::Bool => writeln!(f, "let {} = value.as_bool().unwrap();", name)?,
        Type::Int(ty) => writeln!(f, "let {} = value.as_{}().unwrap()", name, ty)?,
        Type::Array(ty, size) => {
            match size {
                Some(size) => writeln!(f, "let mut {} = [{}::default(); {}];", name, ty, size)?,
                None => writeln!(
                    f,
                    "let mut {} = vec![{}::default(); self.0.recv_u32()? as usize];",
                    name, ty
                )?,
            };
            writeln!(f, "for (i, value) in value.members().enumerate() {{")?;
            from_json_value(f, "x", ty)?;
            writeln!(f, "{}[i] = x;", name)?;
            writeln!(f, "}}")?;
        }
        Type::Struct(struct_) => {
            for (field_name, ty) in &struct_.fields {
                writeln!(f, "{{")?;
                writeln!(f, "let value = value[\"{}\"].unwrap()", field_name)?;
                from_json_value(f, &field_name, ty)?;
                writeln!(f, "}}")?;
            }
            write!(f, "let {} = super::super::{} {{", name, struct_.name)?;
            for (field_name, _) in &struct_.fields {
                write!(f, "{},", field_name)?;
            }
            writeln!(f, "}};")?;
        }
    };
    Ok(())
}

fn to_json_value(f: &mut fmt::Formatter<'_>, name: &str, ty: &Type) -> fmt::Result {
    match ty {
        Type::Bool | Type::Int(_) => writeln!(f, "let value = {}.into()", name)?,
        Type::Array(ty, _) => {
            writeln!(f, "let mut array = vec![]")?;
            writeln!(f, "for i in 0..{}.len() {{", name)?;
            to_json_value(f, &format!("{}[i]", name), ty)?;
            writeln!(f, "array.push(value)")?;
            writeln!(f, "}}")?;
            writeln!(f, "let value = JsonValue::from(array);")?;
        }
        Type::Struct(struct_) => {
            writeln!(f, "let mut object = HashMap::new();")?;
            for (field_name, ty) in &struct_.fields {
                to_json_value(f, &format!("{}.{}", name, field_name), ty)?;
                writeln!(f, "object.insert({}, value);", field_name)?;
            }
            writeln!(f, "let value = JsonValue::from(object);")?;
        }
    };
    Ok(())
}
