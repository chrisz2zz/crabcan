use std::os::fd::RawFd;

use nix::sys::socket::{recv, send, socketpair, MsgFlags, SockFlag};

use crate::errors::Errcode;

pub fn generate_socketpair() -> Result<(RawFd, RawFd), Errcode> {
    match socketpair(
        nix::sys::socket::AddressFamily::Unix,
        nix::sys::socket::SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC,
    ) {
        Ok(res) => Ok(res),
        Err(_) => Err(Errcode::SocketError(0)),
    }
}

pub fn send_boolean(fd: RawFd, boolean: bool) -> Result<(), Errcode> {
    let data: [u8; 1] = [boolean.into()];
    if let Err(e) = send(fd, &data, MsgFlags::empty()) {
        log::error!("Cannot send boolean through socket: {:?}", e);
        return Err(Errcode::SocketError(1));
    }
    Ok(())
}

pub fn recv_boolean(fd: RawFd) -> Result<bool, Errcode> {
    let mut data: [u8; 1] = [0];
    if let Err(e) = recv(fd, &mut data, MsgFlags::empty()) {
        log::error!("Cannot receive boolean from socket: {:?}", e);
        return Err(Errcode::SocketError(2));
    }
    Ok(data[0] == 1)
}
