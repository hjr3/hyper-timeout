extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_service;
extern crate hyper;

use std::time::Duration;
use std::io;

use futures::future::{Either, Future};

use tokio_core::reactor::{Handle, Timeout};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_service::Service;

use hyper::client::Connect;

/// A connector that enforces as connection timeout
#[derive(Debug)]
pub struct TimeoutConnector<T> {
    /// A connector implementing the `Connect` trait
    connector: T,
    /// Handle to be used to set the timeout within tokio's core
    handle: Handle,
    /// Amount of time to wait connecting
    connect_timeout: Duration,
}

impl<T: Connect> TimeoutConnector<T> {
    /// Construct a new TimeoutConnector with a given connector implementing the `Connect` trait
    pub fn new(connector: T, handle: &Handle, timeout: Duration) -> Self {
        TimeoutConnector {
            connector: connector,
            handle: handle.clone(),
            connect_timeout: timeout,
        }
    }
}

impl<T> Service for TimeoutConnector<T>
    where T: Service<Error=io::Error> + 'static,
          T::Response: AsyncRead + AsyncWrite,
          T::Future: Future<Error=io::Error>,
{
    type Request = T::Request;
    type Response = T::Response;
    type Error = T::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let connecting = self.connector.call(req);
        let timeout = Timeout::new(self.connect_timeout, &self.handle).unwrap();

        Box::new(connecting.select2(timeout).then(|res| {
            match res {
                Ok(Either::A((io, _))) => Ok(io),
                Ok(Either::B((_, _))) => {
                    Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "Client timed out while connecting"
                        ))
                }
                Err(Either::A((e, _))) => Err(e),
                Err(Either::B((e, _))) => Err(e),
            }
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
        let connector = TimeoutConnector::new(
            HttpConnector::new(1, &core.handle()),
            &core.handle(),
            Duration::from_millis(1)
        );

        assert_eq!(core.run(connector.connect(url)).unwrap_err().kind(), io::ErrorKind::TimedOut);
    }
}
