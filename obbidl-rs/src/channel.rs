use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{self, Read, Write},
    net::TcpStream,
    rc::{Rc, Weak},
};

pub trait Channel {
    type Error;

    fn recv(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    fn recv_u8(&mut self) -> Result<u8, Self::Error> {
        let mut data = [0; 1];
        self.recv(&mut data)?;
        Ok(data[0])
    }
    fn send_u8(&mut self, data: u8) -> Result<(), Self::Error> {
        self.send(&[data])
    }
}

struct TcpChannel {
    stream: TcpStream,
}

impl Channel for TcpChannel {
    type Error = io::Error;

    fn recv(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.stream.read_exact(data)
    }

    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.stream.write_all(data)
    }
}

pub struct TestChannel {
    send: Weak<RefCell<VecDeque<u8>>>,
    recv: Rc<RefCell<VecDeque<u8>>>,
}

#[derive(Debug, Clone, Copy)]
pub enum TestChannelError {
    NoData,
    Closed,
}

impl Channel for TestChannel {
    type Error = TestChannelError;

    fn recv(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        let mut queue = self.recv.borrow_mut();
        for byte in data {
            *byte = queue.pop_back().ok_or(TestChannelError::NoData)?;
        }
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let rc = self.send.upgrade().ok_or(TestChannelError::Closed)?;
        let mut queue = rc.borrow_mut();
        for byte in data {
            queue.push_front(*byte);
        }
        Ok(())
    }
}

impl TestChannel {
    pub fn new() -> (TestChannel, TestChannel) {
        let a = Rc::new(RefCell::new(VecDeque::new()));
        let b = Rc::new(RefCell::new(VecDeque::new()));

        let a_weak = Rc::downgrade(&a);
        let b_weak = Rc::downgrade(&b);

        (
            TestChannel {
                send: a_weak,
                recv: b,
            },
            TestChannel {
                send: b_weak,
                recv: a,
            },
        )
    }
}
