include!(concat!(env!("OUT_DIR"), "/test.rs"));

use obbidl::channel::TestChannel;
use test::{cli, ser};

fn main() {
    let (client_channel, server_channel) = TestChannel::new();

    let client = cli::S0::new(client_channel);
    let server = ser::S0::new(server_channel);

    client.send_send(&Point { x: 5, y: 2 }).unwrap().finish();

    match server.recv_default().unwrap() {
        ser::S0Response::send { state, param0 } => println!(),
    }
}
