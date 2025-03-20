#[derive(Debug)]
pub enum Error {
    Encode,
    Decode,
    UnixSocket(String),
}

pub mod sync {
    use super::Error;
    use std::{
        io::{Read, Write},
        os::unix::net::{UnixListener, UnixStream},
    };

    use bincode::{de, enc};

    pub struct UnixListenerWrapper {
        listener: UnixListener,
    }

    impl UnixListenerWrapper {
        pub fn bind<P>(path: P) -> Result<Self, Error>
        where
            P: AsRef<std::path::Path>,
        {
            drop(std::fs::remove_file(&path));
            match UnixListener::bind(path) {
                Ok(listener) => Ok(Self { listener }),
                Err(e) => Err(Error::UnixSocket(e.to_string())),
            }
        }

        pub fn loop_accept<F>(&self, mut callback: F) -> Result<(), Error>
        where
            F: FnMut(UnixStreamWrapper) -> Result<bool, Error>,
        {
            loop {
                match self.listener.accept() {
                    Err(e) => println!("failed to connect {e:?}"),
                    Ok((stream, _addr)) => {
                        let stream = UnixStreamWrapper::new(stream);
                        match callback(stream) {
                            Ok(alive) => {
                                if alive {
                                    continue;
                                }
                                return Ok(());
                            }
                            Err(e) => println!("failed to execute callback {e:?}"),
                        }
                    }
                }
            }
        }
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
}

pub mod r#async {
    use super::Error;
    use futures_lite::io::{AsyncReadExt, AsyncWriteExt};

    use async_net::unix::{UnixListener, UnixStream};

    use bincode::{de, enc};

    pub struct UnixListenerWrapper {
        listener: UnixListener,
    }

    impl UnixListenerWrapper {
        pub fn bind<P>(path: P) -> Result<Self, Error>
        where
            P: AsRef<std::path::Path>,
        {
            drop(std::fs::remove_file(&path));
            match UnixListener::bind(path) {
                Ok(listener) => Ok(Self { listener }),
                Err(e) => Err(Error::UnixSocket(e.to_string())),
            }
        }

        pub async fn loop_accept<F>(&self, mut callback: F) -> Result<(), Error>
        where
            F: AsyncFnMut(UnixStreamWrapper) -> bool,
        {
            loop {
                match self.listener.accept().await {
                    Err(e) => println!("failed to connect {e:?}"),
                    Ok((stream, _addr)) => {
                        let stream = UnixStreamWrapper::new(stream);
                        let alive = callback(stream).await;
                        if !alive {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    pub struct UnixStreamWrapper {
        stream: UnixStream,
    }

    impl UnixStreamWrapper {
        pub fn new(stream: UnixStream) -> Self {
            Self { stream }
        }

        pub async fn read<T>(&mut self) -> Result<T, Error>
        where
            T: de::Decode<()>,
        {
            let mut response = vec![];
            drop(self.stream.read_to_end(&mut response).await);
            match bincode::decode_from_slice(&response, bincode::config::standard()) {
                Ok((response, _len)) => Ok(response),
                Err(_) => Err(Error::Decode),
            }
        }

        pub async fn write<E>(&mut self, payload: E) -> Result<(), Error>
        where
            E: enc::Encode,
        {
            let ans = match self
                .stream
                .write_all(&bincode::encode_to_vec(&payload, bincode::config::standard()).unwrap())
                .await
            {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Encode),
            };
            drop(self.stream.shutdown(std::net::Shutdown::Write));
            ans
        }
    }
}
