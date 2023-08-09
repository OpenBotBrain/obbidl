include!(concat!(env!("OUT_DIR"), "/test.rs"));

use obbidl::channel::{Channel, TestChannel};
use test::{c, s};

struct Accumulator {
    total: u32,
}

impl<C: Channel<Error = E>, E> s::S0Receiver<C, E> for Accumulator {
    type Type = (u32, s::S1<C>);

    fn recv_add(mut self, state: s::S0<C>, a: u32) -> Result<Self::Type, E> {
        self.total += a;
        state.recv(self)
    }

    fn recv_subtract(mut self, state: s::S0<C>, b: u32) -> Result<Self::Type, E> {
        self.total -= b;
        state.recv(self)
    }

    fn recv_finish(self, state: s::S1<C>) -> Result<Self::Type, E> {
        Ok((self.total, state))
    }
}

struct Response;

impl<C: Channel<Error = E>, E> c::S1Receiver<C, E> for Response {
    type Type = u32;

    fn recv_total(self, state: c::S2<C>, total: u32) -> Result<Self::Type, E> {
        state.finish();
        Ok(total)
    }
}

fn main() {
    let (client_channel, server_channel) = TestChannel::new();

    let mut client = c::S0::new(client_channel);
    let server = s::S0::new(server_channel);

    for i in 0..100 {
        if i % 3 != 0 {
            client = client.send_add(i * 2).unwrap();
        } else {
            client = client.send_subtract(i).unwrap();
        }
    }
    let client = client.send_finish().unwrap();

    let (total, server) = server.recv(Accumulator { total: 0 }).unwrap();
    server.send_total(total).unwrap().finish();

    let total = client.recv(Response).unwrap();
    println!("{}", total);
}
