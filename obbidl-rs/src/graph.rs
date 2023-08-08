use std::fmt;

use crate::compile::ProtocolFileStateMachines;

#[derive(Debug, Clone)]
pub struct GraphViz<'a>(pub &'a ProtocolFileStateMachines);

impl<'a> fmt::Display for GraphViz<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for protocol in &self.0.protocols {
            writeln!(f, "digraph {{")?;
            writeln!(f, "label=\"{}\"", protocol.name)?;
            for trans in protocol.state_machine.iter_transitions() {
                writeln!(
                    f,
                    "  {} -> {}[label=\"{}\"];",
                    trans.start.0, trans.end.0, trans.msg.label,
                )?;
            }
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}
