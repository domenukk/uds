use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::{UnixStream, UnixDatagram};
use std::io;

use libc::{socket, AF_UNIX, SOCK_STREAM, SOCK_CLOEXEC, close};

use crate::addr::*;
use crate::ancillary::*;

pub trait UnixStreamExt: AsRawFd + FromRawFd + Sized {
    fn local_unix_addr(&self) -> Result<UnixSocketAddr, io::Error> {
        local_addr(self.as_raw_fd())
    }
    fn peer_unix_addr(&self) -> Result<UnixSocketAddr, io::Error> {
        peer_addr(self.as_raw_fd())
    }

    fn connect_to_unix_addr(addr: &UnixSocketAddr) -> Result<Self, io::Error>;
    fn connect_from_to(from: &UnixSocketAddr,  to: &UnixSocketAddr) -> Result<Self, io::Error>;

    fn send_fds(&self,  bytes: &[u8],  fds: &[RawFd]) -> Result<usize, io::Error>;
}

impl UnixStreamExt for UnixStream {
    fn connect_to_unix_addr(addr: &UnixSocketAddr) -> Result<Self, io::Error> {
        let sock = unsafe { socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0) };
        if sock == -1 {
            return Err(io::Error::last_os_error());
        }
        match connect_to(sock, addr) {
            Ok(()) => Ok(unsafe { UnixStream::from_raw_fd(sock) }),
            Err(err) => {
                unsafe { close(sock) };
                Err(err)
            }
        }
    }
    fn connect_from_to(from: &UnixSocketAddr,  to: &UnixSocketAddr) -> Result<Self, io::Error> {
        let sock = unsafe { socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0) };
        if sock == -1 {
            return Err(io::Error::last_os_error());
        }
        match bind_to(sock, from).and_then(|()| connect_to(sock, to) ) {
            Ok(()) => Ok(unsafe { UnixStream::from_raw_fd(sock) }),
            Err(err) => {
                unsafe { close(sock) };
                Err(err)
            }
        }
    }

    fn send_fds(&self,  bytes: &[u8],  fds: &[RawFd]) -> Result<usize, io::Error> {
        send_ancillary(self.as_raw_fd(), None, 0, &[IoSlice::new(bytes)], fds, None)
    }
}



pub trait UnixDatagramExt: AsRawFd + FromRawFd + Sized {
    fn local_unix_addr(&self) -> Result<UnixSocketAddr, io::Error> {
        local_addr(self.as_raw_fd())
    }
    fn peer_unix_addr(&self) -> Result<UnixSocketAddr, io::Error> {
        peer_addr(self.as_raw_fd())
    }
    fn connect_to_unix_addr(&self,  addr: &UnixSocketAddr) -> Result<(), io::Error>;
    fn bind_to_unix_addr(&self,  addr: &UnixSocketAddr) -> Result<(), io::Error>;

    fn send_fds_to(&self,  datagram: &[u8],  fds: &[RawFd],  addr: &UnixSocketAddr)
    -> Result<usize, io::Error>;
}

impl UnixDatagramExt for UnixDatagram {
    fn connect_to_unix_addr(&self,  addr: &UnixSocketAddr) -> Result<(), io::Error> {
        connect_to(self.as_raw_fd(), addr)
    }
    fn bind_to_unix_addr(&self,  addr: &UnixSocketAddr) -> Result<(), io::Error> {
        bind_to(self.as_raw_fd(), addr)
    }

    fn send_fds_to(&self,  datagram: &[u8],  fds: &[RawFd],  addr: &UnixSocketAddr)
    -> Result<usize, io::Error> {
        send_ancillary(self.as_raw_fd(), Some(addr), 0, &[IoSlice::new(datagram)], fds, None)
    }
}
