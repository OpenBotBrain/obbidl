use std::collections::{HashMap, VecDeque};

use crate::{
    ast::{Message, Sequence, Stmt},
    fsm::{StateMachine, Transistion},
};

pub fn compile_seq(seq: &Sequence) -> StateMachine {
    let mut state_machine = StateMachine::new();
    let mut states = HashMap::new();
    let mut queue = VecDeque::new();

    let start = state_machine.new_state();
    queue.push_front((seq.clone(), start));
    states.insert(seq.clone(), start);

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

    state_machine
}

fn seq_may_terminate(seq: &Sequence) -> bool {
    for stmt in &seq.0 {
        if !stmt_may_terminate(stmt) {
            return false;
        }
    }
    true
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
                    let new_seqs = seqs
                        .iter()
                        .take(i)
                        .cloned()
                        .chain(Some(rem_seq).into_iter())
                        .chain(seqs.iter().rev().take(seqs.len() - i - 1).rev().cloned())
                        .collect();
                    trans.push((
                        msg,
                        Sequence(
                            Some(Stmt::Par(new_seqs))
                                .iter()
                                .chain(iter.clone())
                                .cloned()
                                .collect(),
                        ),
                    ))
                }
            }
        }
        Stmt::Inf(seq) => {
            for (msg, rem_seq) in generate_transitions(seq) {
                trans.push((
                    msg,
                    Sequence(
                        rem_seq
                            .0
                            .into_iter()
                            .chain(Some(Stmt::Inf(seq.clone())).into_iter())
                            .chain(iter.clone().cloned())
                            .collect(),
                    ),
                ))
            }
        }
        Stmt::Fin(seq) => {
            for (msg, rem_seq) in generate_transitions(seq) {
                trans.push((
                    msg,
                    Sequence(
                        rem_seq
                            .0
                            .into_iter()
                            .chain(Some(Stmt::Fin(seq.clone())).into_iter())
                            .chain(iter.clone().cloned())
                            .collect(),
                    ),
                ));
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
    use crate::{
        ast::Sequence,
        parser::{parse, parse_maybe},
    };

    use super::generate_transitions;

    #[test]
    fn test_msg_trans() {
        let seq = parse("{ X from C to S; Y from S to C; }").unwrap();
        let trans = generate_transitions(&seq);

        assert_eq!(trans.len(), 1);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse_maybe("X from C to S;").unwrap().unwrap());
        assert_eq!(rem_seq, &parse("{ Y from S to C; }").unwrap());
    }

    #[test]
    fn test_choice_trans() {
        let seq: Sequence =
            parse("{ choice { X from C to S; } or { Y from C to S; Z from S to C; } }").unwrap();
        let trans = generate_transitions(&seq);

        assert_eq!(trans.len(), 2);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse_maybe("X from C to S;").unwrap().unwrap());
        assert_eq!(rem_seq, &parse("{ }").unwrap());

        let (msg, rem_seq) = &trans[1];
        assert_eq!(msg, &parse_maybe("Y from C to S;").unwrap().unwrap());
        assert_eq!(rem_seq, &parse("{ Z from S to C; }").unwrap());
    }

    #[test]
    fn test_par_trans() {
        let seq: Sequence =
            parse("{ par { X from C to S; } and { Y from C to S; } and { Z from C to S; W from S to C; } }")
                .unwrap();
        let trans = generate_transitions(&seq);

        assert_eq!(trans.len(), 3);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse_maybe("X from C to S;").unwrap().unwrap());
        assert_eq!(
            rem_seq,
            &parse("{ par {} and { Y from C to S; } and { Z from C to S; W from S to C; } }")
                .unwrap()
        );

        let (msg, rem_seq) = &trans[1];
        assert_eq!(msg, &parse_maybe("Y from C to S;").unwrap().unwrap());
        assert_eq!(
            rem_seq,
            &parse("{ par { X from C to S; } and {} and { Z from C to S; W from S to C; } }")
                .unwrap()
        );

        let (msg, rem_seq) = &trans[2];
        assert_eq!(msg, &parse_maybe("Z from C to S;").unwrap().unwrap());
        assert_eq!(
            rem_seq,
            &parse("{ par { X from C to S; } and { Y from C to S; } and { W from S to C; } }")
                .unwrap()
        );
    }

    #[test]
    fn test_inf_trans() {
        let seq: Sequence = parse("{ inf { X from C to S; Y from S to C; } }").unwrap();
        let trans = generate_transitions(&seq);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse_maybe("X from C to S;").unwrap().unwrap());
        assert_eq!(
            rem_seq,
            &parse("{ Y from S to C; inf { X from C to S; Y from S to C; } }").unwrap()
        );
    }

    #[test]
    fn test_fin_trans() {
        let seq: Sequence =
            parse("{ fin { X from C to S; Y from C to S; } Z from C to S; }").unwrap();
        let trans = generate_transitions(&seq);

        let (msg, rem_seq) = &trans[0];
        assert_eq!(msg, &parse_maybe("X from C to S;").unwrap().unwrap());
        assert_eq!(
            rem_seq,
            &parse("{ Y from C to S; fin { X from C to S; Y from C to S; } Z from C to S; }")
                .unwrap()
        );

        let (msg, rem_seq) = &trans[1];
        assert_eq!(msg, &parse_maybe("Z from C to S;").unwrap().unwrap());
        assert_eq!(rem_seq, &parse("{ }").unwrap());
    }
}
