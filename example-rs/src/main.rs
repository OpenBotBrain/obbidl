use obbidl_derive::include_obbidl_file;

include_obbidl_file!("test.txt");

use obbidl::channel::{TestChannel, TestChannelError};
use test::{cli, ser};

fn main() {
    thing().unwrap();
}

fn thing() -> Result<(), TestChannelError> {
    let (client_channel, server_channel) = TestChannel::new();

    let client = cli::S0::new(client_channel);
    let server = ser::S0::new(server_channel);

    let client = client.send_a(26)?.send_b(23)?;

    let (server, a) = match server.recv_default()? {
        ser::S0Response::a { state, param0 } => (state, param0),
    };
    let (server, b) = match server.recv_default()? {
        ser::S1Response::b { state, param0 } => (state, param0),
    };
    server.send_c(a + b)?.finish();

    match client.recv_default()? {
        cli::S2Response::c { state, param0 } => {
            state.finish();
            println!("{}", param0)
        }
    }

    Ok(())
}
