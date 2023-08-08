





use std::mem::size_of;
use obbidl::channel::Channel;



mod test {

    mod C {
        



#[must_use]
pub struct S0<C: Channel>(C);






impl<C: Channel<Error = E>, E> S0<C> {
    
    pub fn send_x(mut self, 
    
        a:
        
            
i32

        ,
    
) -> Result<S1<C>, E> {
        self.0.send_u8(0)?;

        
        
        self.0.send(&
i32
::to_be_bytes(a))?;
        
        

        Ok(S1(self.0))
    }
    
    pub fn send_y(mut self, 
    
        b:
        
            
i32

        ,
    
) -> Result<S1<C>, E> {
        self.0.send_u8(1)?;

        
        
        self.0.send(&
i32
::to_be_bytes(b))?;
        
        

        Ok(S1(self.0))
    }
    
}






#[must_use]
pub struct S1<C: Channel>(C);





pub trait S1Receiver<C: Channel<Error = E>, E> {
    type Type;

    
    fn recv_z(self, state: S2<C>, 
    
        x:
        
            
u16

        ,
    
) -> Result<Self::Type, E>;
    
}

impl<C: Channel<Error = E>, E> S1<C> {

    pub fn recv<T>(mut self, receiver: impl S1Receiver<C, E, Type = T>) -> Result<T, E> {
        
        if self.0.recv_u8()? == 0 {
            
            
            let mut bytes = [0; size_of::<
u16
>()];
            self.0.recv(&mut bytes)?;
            let x = 
u16
::from_be_bytes(bytes);
            
            

            return Ok(receiver.recv_z(S2(self.0),
            
                x,
            
            )?)
        }
        
        panic!("invalid message!")
    }

}









#[must_use]
pub struct S2<C: Channel>(C);



impl<C: Channel<Error = E>, E> S2<C> {
    pub fn finish(self) {}
}





impl<C: Channel> S0<C> {
    pub fn new(channel: C) -> S0<C> {
        S0(channel)
    }
}


    }

    mod S {
        



#[must_use]
pub struct S0<C: Channel>(C);





pub trait S0Receiver<C: Channel<Error = E>, E> {
    type Type;

    
    fn recv_x(self, state: S1<C>, 
    
        a:
        
            
i32

        ,
    
) -> Result<Self::Type, E>;
    
    fn recv_y(self, state: S1<C>, 
    
        b:
        
            
i32

        ,
    
) -> Result<Self::Type, E>;
    
}

impl<C: Channel<Error = E>, E> S0<C> {

    pub fn recv<T>(mut self, receiver: impl S0Receiver<C, E, Type = T>) -> Result<T, E> {
        
        if self.0.recv_u8()? == 0 {
            
            
            let mut bytes = [0; size_of::<
i32
>()];
            self.0.recv(&mut bytes)?;
            let a = 
i32
::from_be_bytes(bytes);
            
            

            return Ok(receiver.recv_x(S1(self.0),
            
                a,
            
            )?)
        }
        
        if self.0.recv_u8()? == 1 {
            
            
            let mut bytes = [0; size_of::<
i32
>()];
            self.0.recv(&mut bytes)?;
            let b = 
i32
::from_be_bytes(bytes);
            
            

            return Ok(receiver.recv_y(S1(self.0),
            
                b,
            
            )?)
        }
        
        panic!("invalid message!")
    }

}









#[must_use]
pub struct S1<C: Channel>(C);






impl<C: Channel<Error = E>, E> S1<C> {
    
    pub fn send_z(mut self, 
    
        x:
        
            
u16

        ,
    
) -> Result<S2<C>, E> {
        self.0.send_u8(0)?;

        
        
        self.0.send(&
u16
::to_be_bytes(x))?;
        
        

        Ok(S2(self.0))
    }
    
}






#[must_use]
pub struct S2<C: Channel>(C);



impl<C: Channel<Error = E>, E> S2<C> {
    pub fn finish(self) {}
}





impl<C: Channel> S0<C> {
    pub fn new(channel: C) -> S0<C> {
        S0(channel)
    }
}


    }

}

