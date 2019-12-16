use std::io::{Result, ErrorKind};
use tokio::prelude::{AsyncWrite};
use std::thread::sleep;
use std::time::{Duration};

pub(crate) fn write_patient(writer: &mut impl AsyncWrite, buf: &[u8]) -> Result<()> {
    // TODO Find a better solution for non-blockingly sending a large buffer
    let length = buf.len();
    let mut already_written = 0;

    while already_written < length {
        match writer.write(&buf[already_written..]) {
            Err(e) => {
                match e.kind() {
                    ErrorKind::BrokenPipe => return Err(e),
                    _ => {
                        sleep(Duration::from_millis(1)); // Yes i know magic timing value
                    }
                }
            },
            Ok(written) => {
                already_written += written;
                sleep(Duration::from_micros(1)); // Yes i know magic timing value
            }
        }
    }

    return Ok(());
}
