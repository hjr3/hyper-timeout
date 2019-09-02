extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate tokio_io_timeout;
extern crate hyper;

use std::time::Duration;
use std::io;

use futures::future::{Either, Future};

use tokio_core::reactor::{Handle, Timeout};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_service::Service;
use tokio_io_timeout::TimeoutStream;

use hyper::client::Connect;

/// A connector that enforces as connection timeout
#[derive(Debug)]
pub struct TimeoutConnector<T> {
    /// A connector implementing the `Connect` trait
    connector: T,
    /// Handle to be used to set the timeout within tokio's core
    handle: Handle,
    /// Amount of time to wait connecting
    connect_timeout: Option<Duration>,
    /// Amount of time to wait reading response
    read_timeout: Option<Duration>,
    /// Amount of time to wait writing request
    write_timeout: Option<Duration>,
}

impl<T: Connect> TimeoutConnector<T> {
    /// Construct a new TimeoutConnector with a given connector implementing the `Connect` trait
    pub fn new(connector: T, handle: &Handle) -> Self {
        TimeoutConnector {
            connector: connector,
            handle: handle.clone(),
            connect_timeout: None,
            read_timeout: None,
            write_timeout: None,
        }
    }

    /// Set the timeout for connecting to a URL.
    ///
    /// Default is no timeout.
    #[inline]
    pub fn set_connect_timeout(&mut self, val: Option<Duration>) {
        self.connect_timeout = val;
    }

    /// Set the timeout for the response.
    ///
    /// Default is no timeout.
    #[inline]
    pub fn set_read_timeout(&mut self, val: Option<Duration>) {
        self.read_timeout = val;
    }

    /// Set the timeout for the request.
    ///
    /// Default is no timeout.
    #[inline]
    pub fn set_write_timeout(&mut self, val: Option<Duration>) {
        self.write_timeout = val;
    }
}

impl<T> Service for TimeoutConnector<T>
where
    T: Service<Error = io::Error> + 'static,
    T::Response: AsyncRead + AsyncWrite,
    T::Future: Future<Error = io::Error>,
{
    type Request = T::Request;
    type Response = TimeoutStream<T::Response>;
    type Error = T::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let read_timeout = self.read_timeout.clone();
        let write_timeout = self.write_timeout.clone();
        let connecting = self.connector.call(req);

        if self.connect_timeout.is_none() {
            return Box::new(connecting.map(move |io| {
                let mut tm = TimeoutStream::new(io);
                tm.set_read_timeout(read_timeout);
                tm.set_write_timeout(write_timeout);
                tm
            }));
        }

        let connect_timeout = self.connect_timeout.expect("Connect timeout should be set");
        let timeout = Timeout::new(connect_timeout, &self.handle).unwrap();

        Box::new(connecting.select2(timeout).then(move |res| match res {
            Ok(Either::A((io, _))) => {
                let mut tm = TimeoutStream::new(io);
                tm.set_read_timeout(read_timeout);
                tm.set_write_timeout(write_timeout);
                Ok(tm)
            }
            Ok(Either::B((_, _))) => {
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Client timed out while connecting",
                ))
            }
            Err(Either::A((e, _))) => Err(e),
            Err(Either::B((e, _))) => Err(e),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::time::Duration;
    use tokio_core::reactor::Core;
    use hyper::client::{Connect, HttpConnector};
    use super::TimeoutConnector;

    #[test]
    fn test_timeout_connector() {
        let mut core = Core::new().unwrap();
        // 10.255.255.1 is a not a routable IP address
        let url = "http://10.255.255.1".parse().unwrap();
        let mut connector =
            TimeoutConnector::new(HttpConnector::new(1, &core.handle()), &core.handle());
        connector.set_connect_timeout(Some(Duration::from_millis(1)));

        match core.run(connector.connect(url)) {
            Err(e) => {
                assert_eq!(e.kind(), io::ErrorKind::TimedOut);
            }
            _ => panic!("Expected timeout error"),
        }
    }
}
