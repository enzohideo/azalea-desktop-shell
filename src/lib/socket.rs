use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use bincode::{de, enc};

#[derive(Debug)]
pub enum Error {
    Encode,
    Decode,
}

pub struct UnixStreamWrapper {
    stream: UnixStream,
}

impl UnixStreamWrapper {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
    }

    pub fn read<T>(&mut self) -> Result<T, Error>
    where
        T: de::Decode<()>,
    {
        let mut response = vec![];
        drop(self.stream.read_to_end(&mut response));
        match bincode::decode_from_slice(&response, bincode::config::standard()) {
            Ok((response, _len)) => Ok(response),
            Err(_) => Err(Error::Decode),
        }
    }

    pub fn write<E>(&mut self, payload: E) -> Result<(), Error>
    where
        E: enc::Encode,
    {
        let ans = match self
            .stream
            .write_all(&bincode::encode_to_vec(&payload, bincode::config::standard()).unwrap())
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Encode),
        };
        drop(self.stream.shutdown(std::net::Shutdown::Write));
        ans
    }
}
