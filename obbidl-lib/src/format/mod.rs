use std::fmt;

use crate::validate::Message;

pub mod binary;
pub mod json;

pub trait Format {
    fn send_message(
        f: &mut fmt::Formatter<'_>,
        messages: &Message,
        insert_tag: bool,
    ) -> fmt::Result;
    fn recv_messages(f: &mut fmt::Formatter<'_>, messages: &[Message]) -> fmt::Result;
}
