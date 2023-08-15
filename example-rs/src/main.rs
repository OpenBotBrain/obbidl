include_obbidl_file!("test.txt");

use obbidl::channel::TestChannel;
use obbidl_derive::include_obbidl_file;
use test::{cli, ser};

fn main() {
    let (client_channel, server_channel) = TestChannel::new();

    let client = cli::S0::new(client_channel);
    let server = ser::S0::new(server_channel);

    server.send_send(&Point { x: 5, y: 2 }).unwrap().finish();

    match client.recv_default().unwrap() {
        cli::S0Response::send { state, param0 } => println!(),
    }
}
