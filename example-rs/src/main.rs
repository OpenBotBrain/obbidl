include!(concat!(env!("OUT_DIR"), "/test.rs"));

use obbidl::channel::{Channel, TestChannel};
use test::{c, s};

struct Accumulator {
    a: Vec<u32>,
    b: Vec<u32>,
}

impl<C: Channel<Error = E>, E> s::S0Receiver<C, E> for Accumulator {
    type Type = ((Vec<u32>, Vec<u32>), s::S1<C>);

    fn recv_a(mut self, state: s::S0<C>, a: u32) -> Result<Self::Type, E> {
        self.a.push(a);
        state.recv(self)
    }

    fn recv_b(mut self, state: s::S0<C>, b: u32) -> Result<Self::Type, E> {
        self.b.push(b);
        state.recv(self)
    }

    fn recv_finish(self, state: s::S1<C>) -> Result<Self::Type, E> {
        Ok(((self.a, self.b), state))
    }
}

struct Response;

impl<C: Channel<Error = E>, E> c::S1Receiver<C, E> for Response {
    type Type = (Vec<u32>, Vec<u32>);

    fn recv_ret(self, state: c::S2<C>, a: &[u32], b: &[u32]) -> Result<Self::Type, E> {
        state.finish();
        Ok((a.iter().copied().collect(), b.iter().copied().collect()))
    }
}

fn main() {
    let (client_channel, server_channel) = TestChannel::new();

    let mut client = c::S0::new(client_channel);
    let server = s::S0::new(server_channel);

    for i in 0..100 {
        if i % 3 != 0 {
            client = client.send_a(i * 2).unwrap();
        } else {
            client = client.send_b(i).unwrap();
        }
    }
    let client = client.send_finish().unwrap();

    let ((a, b), server) = server
        .recv(Accumulator {
            a: vec![],
            b: vec![],
        })
        .unwrap();
    let server = server.send_ret(&a, &b).unwrap();
    server.finish();

    let total = client.recv(Response).unwrap();
    println!("{:?}", total);
}
