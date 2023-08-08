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
