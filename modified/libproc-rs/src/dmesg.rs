extern crate libproc;
extern crate libc;

use std::io::Write;
use libproc::libproc::kmesg_buffer;

fn main() {
    if kmesg_buffer::am_root() {
        match kmesg_buffer::kmsgbuf() {
            Ok(message) => println!("{}", message),
            Err(err) => writeln!(&mut std::io::stderr(), "Error: {}", err).unwrap()
        }
    } else {
        writeln!(&mut std::io::stderr(), "Must be run as root").unwrap()
    }
}