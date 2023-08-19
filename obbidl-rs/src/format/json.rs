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
            writeln!(f, "object.insert(\"state\", \"{}\");", message.label)?;
        }
        for (name, ty) in &message.payload.items {
            to_json_value(f, name, ty)?;
            writeln!(f, "object.insert(\"{}\", value);", name)?;
        }

        Ok(())
    }

    fn recv_messages(f: &mut fmt::Formatter<'_>, messages: &[Message]) -> fmt::Result {
        Ok(())
    }
}

fn from_json_value(f: &mut fmt::Formatter<'_>, name: &str, ty: &Type) -> fmt::Result {
    match ty {
        Type::Bool => writeln!(f, "let {} = value.as_bool().unwrap();", name),
        Type::Int(ty) => writeln!(f, "let {} = value.as_{}().unwrap()", name, ty),
        Type::Array(_, _) => todo!(),
        Type::Struct(_) => todo!(),
    }
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
