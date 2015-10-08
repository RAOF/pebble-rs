extern crate mio;
extern crate nix;
extern crate libc;

mod ffi;

use std::io::{Read, Write, Result};
use std::ffi::CString;
use std::mem;

use std::os::unix::prelude::AsRawFd;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Default)]
pub struct BtAddr(pub [u8; 6]);

impl BtAddr {
    pub fn any () -> BtAddr {
        BtAddr ([0, 0, 0, 0, 0, 0])
    }

    pub fn from_string(addr : &str) -> Option<BtAddr> {
        let mut parsed_address : BtAddr = Self::any();
        match CString::new(addr) {
            Ok(a) => {
                if unsafe { ffi::str2ba(a.as_ptr(), &mut parsed_address) } >= 0 {
                    Some(parsed_address)
                }
                else {
                    None
                }
            }
            Err(_) => None
        }
    }

    pub fn to_string(&self) -> String {
        unsafe {
            let ffi_buffer = CString::from_vec_unchecked(vec![0u8; 17]);
            ffi::ba2str(&self, ffi_buffer.as_ptr());
            String::from(ffi_buffer.to_str().unwrap())
        }
    }
}

#[repr(C)]
#[derive(Copy, Debug, Clone)]
struct sockaddr_rc {
    rc_family : libc::sa_family_t,
    rc_bdaddr : BtAddr,
    rc_channel : u8
}

pub enum BluetoothProtocol {
    rfcomm = BTPROTO_RFCOMM
}

pub struct BluetoothSocket {
    io : mio::Io
}

const AF_BLUETOOTH : i32 = 31;

const BTPROTO_L2CAP : isize = 0;
const BTPROTO_HCI : isize = 1;
const BTPROTO_SCO : isize = 2;
const BTPROTO_RFCOMM : isize = 3;
const BTPROTO_BNEP : isize = 4;
const BTPROTO_CMTP : isize = 5;
const BTPROTO_HIDP : isize = 6;
const BTPROTO_AVDTP : isize = 7;

impl BluetoothSocket {
    fn new(proto : BluetoothProtocol) -> nix::Result<BluetoothSocket> {
        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_STREAM, proto as i32) };

        if fd < 0 {
            Err(nix::Error::last())
        } else {
            Ok(From::from(mio::Io::from_raw_fd(fd)))
        }
    }

    fn connect(&mut self, addr: &BtAddr) -> nix::Result<()> {
        let full_address : sockaddr_rc = sockaddr_rc { rc_family : AF_BLUETOOTH as u16,
            rc_bdaddr : *addr,
            rc_channel : 0
        };

        if unsafe { libc::connect(self.io.as_raw_fd(), &full_address as *const _ as *const libc::sockaddr, mem::size_of::<sockaddr_rc>() as u32) } < 0 {
            Err(nix::Error::last())
        } else {
            Ok(())
        }
    }
}

impl From<mio::Io> for BluetoothSocket {
    fn from(io : mio::Io) -> BluetoothSocket {
        BluetoothSocket { io : io }
    }
}

impl mio::Evented for BluetoothSocket {
    fn register(&self, selector: &mut mio::Selector, token: mio::Token, interest: mio::EventSet, opts: mio::PollOpt) -> Result<()> {
        self.io.register(selector, token, interest, opts)
    }

    fn reregister(&self, selector: &mut mio::Selector, token: mio::Token, interest: mio::EventSet, opts: mio::PollOpt) -> Result<()> {
        self.io.reregister(selector, token, interest, opts)
    }

    fn deregister(&self, selector: &mut mio::Selector) -> Result<()> {
        self.io.deregister(selector)
    }
}

impl Read for BluetoothSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.io.read(buf)
    }
}

impl Write for BluetoothSocket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.io.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.io.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test()]
    fn btaddr_from_string() {

        let addr = BtAddr::any();
        assert_eq!(addr.to_string(), "00:00:00:00:00:00");

        match BtAddr::from_string("addr : String") {
            Some(_) => panic!("Unexpectedly succeeded"),
            None => ()
        }

        match BtAddr::from_string(&super::BtAddr::any().to_string()) {
            Some(_) => (),
            None => panic!("Unexpectedly failed!")
        }
    }

    #[test()]
    fn creates_rfcomm_socket() {
        BluetoothSocket::new(BluetoothProtocol::rfcomm).unwrap();
    }
}
