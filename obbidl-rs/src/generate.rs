use crate::{ast::Payload, fsm::State, to_fsm::ProtocolStateMachine};

struct Trans {
    start: State,
    end: State,
    label: String,
    id: u32,
    payload: Payload,
}

trait Target {
    fn emit_state(output: &mut String, state: State);
    fn emit_send(output: &mut String, trans: Trans);
    fn emit_recv(output: &mut String, trans: Trans);
}

fn generate_bindings<T: Target>(protocol: ProtocolStateMachine, role: &str) -> String {
    let mut output = String::new();

    for state in protocol.state_machine.iter_states() {
        T::emit_state(&mut output, state);
    }

    // for trans in protocol.state_machine.iter_transitions() {
    //     T::emit_trans(&mut output, trans);
    // }

    output
}

struct Rust;

impl Target for Rust {
    fn emit_state(output: &mut String, state: State) {
        output.push_str(&format!("struct {};\n", state.name()));
    }

    fn emit_send(output: &mut String, trans: Trans) {
        todo!()
    }

    fn emit_recv(output: &mut String, trans: Trans) {
        todo!()
    }
}
