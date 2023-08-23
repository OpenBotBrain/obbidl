use std::fmt;

use crate::compile::ProtocolFileStateMachines;

#[derive(Debug, Clone)]
pub struct GraphViz<'a>(pub &'a ProtocolFileStateMachines);

impl<'a> fmt::Display for GraphViz<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for protocol in &self.0.protocols {
            writeln!(f, "digraph {{")?;
            writeln!(f, "  label=\"{}\"", protocol.inner.name)?;
            for trans in protocol.inner.state_machine.iter_transitions() {
                writeln!(
                    f,
                    "  {} -> {}[label=\"{}\"];",
                    trans.start.0, trans.end.0, trans.msg.inner.label,
                )?;
            }
            writeln!(f, "}}")?;
        }
        Ok(())
    }
}
