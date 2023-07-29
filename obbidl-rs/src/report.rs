use std::fmt;

pub trait Report {
    type T;

    fn report(self) -> Self::T;
}

impl<T, E: fmt::Display> Report for Result<T, E> {
    type T = T;

    fn report(self) -> Self::T {
        match self {
            Ok(ty) => ty,
            Err(err) => {
                println!("{}", err);
                panic!()
            }
        }
    }
}
