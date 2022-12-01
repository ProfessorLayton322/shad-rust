#![forbid(unsafe_code)]

use std::net::{TcpListener, TcpStream};
use std::thread;

use std::borrow::Borrow;
use std::io::copy;
use std::io::Result;
use std::sync::{Arc, Mutex};

pub struct StreamWrapper {
    inner: Arc<Mutex<TcpStream>>,
}

impl std::io::Read for StreamWrapper {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mtx: &Mutex<TcpStream> = self.inner.borrow();
        mtx.lock().unwrap().read(buf)
    }
}

impl std::io::Write for StreamWrapper {
    fn flush(&mut self) -> Result<()> {
        let mtx: &Mutex<TcpStream> = self.inner.borrow();
        mtx.lock().unwrap().flush()
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mtx: &Mutex<TcpStream> = self.inner.borrow();
        mtx.lock().unwrap().write(buf)
    }
}

pub fn two_arcs(stream: TcpStream) -> (StreamWrapper, StreamWrapper) {
    let arc = Arc::new(Mutex::new(stream));
    (
        StreamWrapper { inner: arc.clone() },
        StreamWrapper { inner: arc },
    )
}

pub fn run_proxy(port: u32, destination: String) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for incoming in listener.incoming() {
        let client = incoming.unwrap();
        let server = TcpStream::connect(destination.as_str()).unwrap();
        let (mut client1, mut client2) = two_arcs(client);
        let (mut server1, mut server2) = two_arcs(server);
        thread::spawn(move || {
            copy(&mut client1, &mut server1).unwrap();
        });
        thread::spawn(move || {
            copy(&mut server2, &mut client2).unwrap();
        });
    }
}
