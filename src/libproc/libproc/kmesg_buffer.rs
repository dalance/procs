extern crate libc;
extern crate errno;

use self::libc::{uint32_t, c_int};

use std::{ptr, mem, env};
use std::fmt;

use libproc::proc_pid;

#[cfg(test)]
use std::io;
#[cfg(test)]
use std::io::Write;

// See https://opensource.apple.com/source/xnu/xnu-1456.1.26/bsd/sys/msgbuf.h
const MAX_MSG_BSIZE : c_int = (1*1024*1024);
const MSG_MAGIC : c_int = 0x063061;

// See /usr/include/sys/msgbuf.h on your Mac.
#[repr(C)]
struct MessageBuffer {
    pub msg_magic : c_int,
    pub msg_size : c_int,
    pub msg_bufx : c_int,      // write pointer
    pub msg_bufr : c_int,      // read pointer
    pub msg_bufc : * mut u8     // buffer
}

impl Default for MessageBuffer {
    fn default() -> MessageBuffer {
        MessageBuffer {
            msg_magic : 0,
            msg_size : 0,
            msg_bufx : 0,
            msg_bufr : 0,
            msg_bufc : ptr::null_mut() as * mut u8
        }
    }
}

impl fmt::Debug for MessageBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MessageBuffer {{ magic: 0x{:x}, size: {}, bufx: {}}}", self.msg_magic, self.msg_size, self.msg_bufx)
    }
}

// this extern block links to the libproc library
// Original signatures of functions can be found at http://opensource.apple.com/source/Libc/Libc-594.9.4/darwin/libproc.c
#[link(name = "proc", kind = "dylib")]
extern {
    fn proc_kmsgbuf(buffer : *mut MessageBuffer, buffersize : uint32_t) -> c_int;
}

/// Get upto buffersize bytes from the the kernel message buffer - as used by dmesg
/// extern crate libproc;
/// extern crate libc;
///
/// use std::str;
/// use std::io::Write;
/// use libproc::libproc::kmesg_buffer;
///
/// fn main() {
///     if kmesg_buffer::am_root() {
///         match kmesg_buffer::kmsgbuf() {
///             Ok(message) => println!("{}", message),
///             Err(err) => writeln!(&mut std::io::stderr(), "Error: {}", err).unwrap()
///         }
///     } else {
///         writeln!(&mut std::io::stderr(), "Must be run as root").unwrap()
///     }
// See http://opensource.apple.com//source/system_cmds/system_cmds-336.6/dmesg.tproj/dmesg.c
pub fn kmsgbuf() -> Result<String, String> {
    let mut message_buffer : MessageBuffer = Default::default();
    let ret: i32;

    unsafe {
        ret = proc_kmsgbuf(&mut message_buffer, mem::size_of::<MessageBuffer>() as u32);
    }

    if ret <= 0 {
        Err(proc_pid::get_errno_with_message(ret))
    } else
    {
        if message_buffer.msg_magic != MSG_MAGIC {
            println!("Message buffer: {:?}", message_buffer);
            Err(format!("The magic number 0x{:x} is incorrect", message_buffer.msg_magic))
        } else {
            // Avoid starting beyond the end of the buffer
            if message_buffer.msg_bufx >= MAX_MSG_BSIZE {
                message_buffer.msg_bufx = 0;
            }
            let mut output : Vec<u8> = Vec::new();

            // The message buffer is circular; start at the read pointer, and go to the write pointer - 1.
            unsafe {
                let mut ch : u8;
//                let newl : bool = false;
//                let skip : bool = false;
                let mut p : * mut u8 = message_buffer.msg_bufc.offset(message_buffer.msg_bufx as isize);
                let ep : * mut u8 = message_buffer.msg_bufc.offset((message_buffer.msg_bufx - 1) as isize);
//                let buf : [u8; 5];

                while p != ep {
                    // If at the end, then loop around to the start
                    // TODO should use actual size (from struct element) - not the max size??
                    if p == message_buffer.msg_bufc.offset(MAX_MSG_BSIZE as isize) {
                        p = message_buffer.msg_bufc;
                    }

                    ch = *p;

                    /* Skip "\n<.*>" syslog sequences.
                    if skip {
                        if ch == '>' {
                            newl = skip = false;
                        }
                        continue;
                    }

                    if newl && ch == '<' {
                        skip = true;
                        continue;
                    }

                    if ch == '\0' {
                        continue;
                    }

                    newl = ch == '\n';

//                    (void)vis(buf, ch, 0, 0);

                    if buf[1] == 0 {
                        output.append(buf[0]);
                    } else {
                        output.append("%s", buf);
                    }
                    */

                    output.push(ch);
                    p = p.offset(1);
                }

                Ok(String::from_utf8(output).unwrap())
            }
        }
    }
}

pub fn am_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

// If you want this test to actually test something, then you need to run as root 'sudo cargo test'
#[test]
fn kmessagebuffer_test() {
    if am_root() {
        match kmsgbuf() {
            Ok(buffer) => println!("Buffer: {:?}", buffer),
            Err(message) => assert!(false, message)
        }
    } else {
        writeln!(&mut io::stdout(), "test libproc::kmesg_buffer::kmessagebuffer_test ... skipped as it needs to be run as root").unwrap();
    }
}
