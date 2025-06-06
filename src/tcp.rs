use std::{
    io,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::atomic::AtomicBool,
};

pub struct CancellableTcpListner {
    inner: TcpListener,
    is_cancelled: AtomicBool,
}

pub struct Incoming<'a> {
    listner: &'a CancellableTcpListner,
}

impl CancellableTcpListner {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<CancellableTcpListner> {
        let listner = TcpListener::bind(addr)?;
        listner.set_nonblocking(false)?;
        Ok(CancellableTcpListner {
            inner: listner,
            is_cancelled: AtomicBool::new(false),
        })
    }

    pub fn cancel(&self) -> io::Result<()> {
        self.is_cancelled
            .store(true, std::sync::atomic::Ordering::Release);
        let addr = self.inner.local_addr()?;
        let _ = TcpStream::connect(addr);
        Ok(())
    }
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listner: self }
    }
}

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<TcpStream>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.listner.inner.accept() {
            Ok((stream, _addr)) => {
                let is_cancelled = self
                    .listner
                    .is_cancelled
                    .load(std::sync::atomic::Ordering::Acquire);
                if is_cancelled { None } else { Some(Ok(stream)) }
            }
            Err(e) => {
                let is_cancelled = self
                    .listner
                    .is_cancelled
                    .load(std::sync::atomic::Ordering::Acquire);
                if is_cancelled { None } else { Some(Err(e)) }
            }
        }
    }
}
