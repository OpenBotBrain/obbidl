use std::collections::{HashMap, VecDeque};

use crate::{
    ast::{Message, Protocol, Sequence, Stmt},
    fsm::{StateMachine, Transistion},
};

pub struct ProtocolStateMachine {
    pub name: String,
    pub roles: Vec<String>,
    pub state_machine: StateMachine,
}

static DEFAULT_ROLES: &[&str] = &["C", "S"];

pub fn compile_protocol_def(protocol: &Protocol) -> ProtocolStateMachine {
    let mut state_machine = StateMachine::new();
    let mut states = HashMap::new();
    let mut queue = VecDeque::new();

    let start = state_machine.new_state();
    queue.push_front((protocol.seq.clone(), start));
    states.insert(protocol.seq.clone(), start);

    while let Some((seq, start)) = queue.pop_back() {
        for (msg, seq) in generate_transitions(&seq) {
            let end = *states.entry(seq.clone()).or_insert_with(|| {
                let state = state_machine.new_state();
                queue.push_front((seq, state));
                state
            });

            state_machine.add_transition(Transistion {
                start,
                end,
                label: msg.label,
                payload: msg.payload,
            })
        }
    }

    ProtocolStateMachine {
        name: protocol.name.clone(),
        roles: protocol
            .roles
            .clone()
            .unwrap_or_else(|| DEFAULT_ROLES.iter().map(|role| role.to_string()).collect()),
        state_machine,
    }
}

fn seq_may_terminate(seq: &Sequence) -> bool {
    seq.0.iter().all(stmt_may_terminate)
}

fn stmt_may_terminate(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Message(_) => false,
        Stmt::Choice(seqs) => seqs.iter().any(seq_may_terminate),
        Stmt::Par(seqs) => seqs.iter().all(seq_may_terminate),
        Stmt::Fin(_) => true,
        Stmt::Inf(_) => false,
    }
}

pub fn generate_transitions(seq: &Sequence) -> Vec<(Message, Sequence)> {
    let mut iter = seq.0.iter();
    let Some(stmt) = iter.next() else {
        return vec![];
    };
    let mut trans = vec![];
    match stmt {
        Stmt::Message(msg) => trans.push((msg.clone(), Sequence(iter.clone().cloned().collect()))),
        Stmt::Choice(seqs) => {
            for seq in seqs {
                for (msg, rem_seq) in generate_transitions(seq) {
                    trans.push((msg, rem_seq))
                }
            }
        }
        Stmt::Par(seqs) => {
            for (i, seq) in seqs.iter().enumerate() {
                for (msg, rem_seq) in generate_transitions(seq) {
                    let mut new_seqs = vec![];
                    new_seqs.extend(seqs.iter().take(i).cloned());
                    new_seqs.push(rem_seq);
                    new_seqs.extend(seqs.iter().rev().take(seqs.len() - i - 1).rev().cloned());

                    let mut stmts = vec![Stmt::Par(new_seqs)];
                    stmts.extend(iter.clone().cloned());

                    trans.push((msg, Sequence(stmts)))
                }
            }
        }
        Stmt::Inf(seq) => {
            for (msg, rem_seq) in generate_transitions(seq) {
                let mut stmts = vec![];
                stmts.extend(rem_seq.0);
                stmts.push(Stmt::Inf(seq.clone()));
                stmts.extend(iter.clone().cloned());
                trans.push((msg, Sequence(stmts)))
            }
        }
        Stmt::Fin(seq) => {
            for (msg, rem_seq) in generate_transitions(seq) {
                let mut stmts = vec![];
                stmts.extend(rem_seq.0);
                stmts.push(Stmt::Fin(seq.clone()));
                stmts.extend(iter.clone().cloned());
                trans.push((msg, Sequence(stmts)));
            }
        }
    }
    if stmt_may_terminate(stmt) {
        let rem_seq = Sequence(iter.cloned().collect());
        for (msg, rem_seq) in generate_transitions(&rem_seq) {
            trans.push((msg, rem_seq))
        }
    }
    trans
}

#[cfg(test)]
mod tests {
    use crate::{ast::Sequence, parser::parse, report::Report};

    use super::generate_transitions;

    #[test]
    fn test_msg_trans() {
        let seq = parse("{ X from C to S; Y from S to C; }").report();
        let trans = generate_transitions(&seq);

        assert_eq!(trans.len(), 1);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse("X from C to S;").report());
        assert_eq!(rem_seq, &parse("{ Y from S to C; }").report());
    }

    #[test]
    fn test_choice_trans() {
        let seq: Sequence =
            parse("{ choice { X from C to S; } or { Y from C to S; Z from S to C; } }").report();
        let trans = generate_transitions(&seq);

        assert_eq!(trans.len(), 2);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse("X from C to S;").report());
        assert_eq!(rem_seq, &parse("{ }").report());

        let (msg, rem_seq) = &trans[1];
        assert_eq!(msg, &parse("Y from C to S;").report());
        assert_eq!(rem_seq, &parse("{ Z from S to C; }").report());
    }

    #[test]
    fn test_par_trans() {
        let seq: Sequence =
            parse("{ par { X from C to S; } and { Y from C to S; } and { Z from C to S; W from S to C; } }")
                .report();
        let trans = generate_transitions(&seq);

        assert_eq!(trans.len(), 3);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse("X from C to S;").report());
        assert_eq!(
            rem_seq,
            &parse("{ par {} and { Y from C to S; } and { Z from C to S; W from S to C; } }")
                .report()
        );

        let (msg, rem_seq) = &trans[1];
        assert_eq!(msg, &parse("Y from C to S;").report());
        assert_eq!(
            rem_seq,
            &parse("{ par { X from C to S; } and {} and { Z from C to S; W from S to C; } }")
                .report()
        );

        let (msg, rem_seq) = &trans[2];
        assert_eq!(msg, &parse("Z from C to S;").report());
        assert_eq!(
            rem_seq,
            &parse("{ par { X from C to S; } and { Y from C to S; } and { W from S to C; } }")
                .report()
        );
    }

    #[test]
    fn test_inf_trans() {
        let seq: Sequence = parse("{ inf { X from C to S; Y from S to C; } }").report();
        let trans = generate_transitions(&seq);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse("X from C to S;").report());
        assert_eq!(
            rem_seq,
            &parse("{ Y from S to C; inf { X from C to S; Y from S to C; } }").report()
        );
    }

    #[test]
    fn test_fin_trans() {
        let seq: Sequence =
            parse("{ fin { X from C to S; Y from C to S; } Z from C to S; }").report();
        let trans = generate_transitions(&seq);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse("X from C to S;").report());
        assert_eq!(
            rem_seq,
            &parse("{ Y from C to S; fin { X from C to S; Y from C to S; } Z from C to S; }")
                .report()
        );

        let (msg, rem_seq) = &trans[1];
        assert_eq!(msg, &parse("Z from C to S;").report());
        assert_eq!(rem_seq, &parse("{ }").report());
    }
}
