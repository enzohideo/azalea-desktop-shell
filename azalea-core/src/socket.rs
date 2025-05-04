#[derive(Debug)]
pub enum Error {
    Read,
    Write,
    UnixSocket(String),
}

pub mod sync {
    use crate::log;

    use super::Error;
    use std::{
        io::{Read, Write},
        os::unix::net::{UnixListener, UnixStream},
    };

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
                    Err(e) => log::warning!("failed to connect {e:?}"),
                    Ok((stream, _addr)) => {
                        let stream = UnixStreamWrapper::new(stream);
                        match callback(stream) {
                            Ok(alive) => {
                                if alive {
                                    continue;
                                }
                                return Ok(());
                            }
                            Err(e) => log::warning!("failed to execute callback {e:?}"),
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

        pub fn connect<P>(path: P) -> Result<Self, Error>
        where
            P: AsRef<std::path::Path>,
        {
            match std::os::unix::net::UnixStream::connect(path) {
                Ok(stream) => Ok(UnixStreamWrapper::new(stream)),
                Err(e) => Err(Error::UnixSocket(e.to_string())),
            }
        }

        pub fn read<T>(&mut self) -> Result<T, Error>
        where
            T: serde::de::DeserializeOwned,
        {
            let mut response = vec![];
            drop(self.stream.read_to_end(&mut response));
            match serde_json::from_slice(&response) {
                Ok(response) => Ok(response),
                Err(_) => Err(Error::Read),
            }
        }

        pub fn write<E>(&mut self, payload: E) -> Result<(), Error>
        where
            E: serde::Serialize,
        {
            let ans = match self
                .stream
                .write_all(&serde_json::to_vec(&payload).unwrap())
            {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Write),
            };
            drop(self.stream.shutdown(std::net::Shutdown::Write));
            ans
        }
    }
}

pub mod r#async {
    use crate::log;

    use super::Error;
    use futures_lite::io::{AsyncReadExt, AsyncWriteExt};

    use async_net::unix::{UnixListener, UnixStream};

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

        pub async fn loop_accept<F>(&self, mut callback: F)
        where
            F: AsyncFnMut(UnixStreamWrapper) -> bool,
        {
            loop {
                match self.listener.accept().await {
                    Err(e) => log::warning!("failed to connect {e:?}"),
                    Ok((stream, _addr)) => {
                        let stream = UnixStreamWrapper::new(stream);
                        let alive = callback(stream).await;
                        if !alive {
                            return;
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
            T: serde::de::DeserializeOwned,
        {
            let mut response = vec![];
            drop(self.stream.read_to_end(&mut response).await);
            match serde_json::from_slice(&response) {
                Ok(response) => Ok(response),
                Err(_) => Err(Error::Read),
            }
        }

        pub async fn write<E>(&mut self, payload: E) -> Result<(), Error>
        where
            E: serde::Serialize,
        {
            let ans = match self
                .stream
                .write_all(&serde_json::to_vec(&payload).unwrap())
                .await
            {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::Write),
            };
            drop(self.stream.shutdown(std::net::Shutdown::Write));
            ans
        }
    }
}
