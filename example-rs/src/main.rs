use std::convert::Infallible;

include!(concat!(env!("OUT_DIR"), "/test.txt"));

use obbidl::channel::Channel;
use test::S::{S0Receiver, S0, S1, S2};

struct DummyChannel;

impl Channel for DummyChannel {
    type Error = Infallible;

    fn recv(&mut self, _: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn send(&mut self, _: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
enum Res {
    X,
    Y,
}

struct Receiver;

impl<C: Channel<Error = E>, E> S0Receiver<C, E> for Receiver {
    type Type = (Res, S2<C>);

    fn recv_x(self, state: S1<C>, a: i32) -> Result<Self::Type, E> {
        Ok((Res::X, state.send_z(0)?))
    }

    fn recv_y(self, state: S1<C>, b: i32) -> Result<Self::Type, E> {
        Ok((Res::Y, state.send_z(0)?))
    }
}

fn main() {
    let start = S0::new(DummyChannel);
    let (res, state) = start.recv(Receiver).unwrap();
    state.finish();
    println!("{:?}", res);
}
