use no_std_io::io;
use spin::mutex::Mutex;
use vex_sdk::{vexSerialReadChar, vexSerialWriteBuffer, vexSerialWriteFree};

const STDIO_CHANNEL: u32 = 1;

// TODO: Add Python input module using this + readline implementation
pub static STDIN: Mutex<Stdin> = Mutex::new(Stdin(()));
pub static STDOUT: Mutex<Stdout> = Mutex::new(Stdout(()));

pub struct Stdin(());
pub struct Stdout(());

impl Stdout {
    const BUFFER_SIZE: usize = 2048;
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut iterator = buf.iter_mut();

        let mut byte: i32;
        let mut written: usize = 0;

        // Little bit cursed, but hey it gets the job done...
        while {
            byte = unsafe { vexSerialReadChar(STDIO_CHANNEL) };
            byte != -1
        } {
            if let Some(next) = iterator.next() {
                *next = byte as u8;
                written += 1;
            } else {
                return Ok(written);
            }
        }

        Ok(written)
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written =
            unsafe { vexSerialWriteBuffer(STDIO_CHANNEL, buf.as_ptr(), buf.len() as u32) };

        if written == -1 {
            return Err(io::Error::new(io::ErrorKind::Other, "internal write error"));
        }

        self.flush()?;

        Ok(written as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            while vexSerialWriteFree(STDIO_CHANNEL) < Self::BUFFER_SIZE as i32 {
                vex_sdk::vexTasksRun();
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn print_inner(args: core::fmt::Arguments<'_>) {
    use alloc::format;

    use io::Write;

    // Panic on print if stdout is not available. While this is less than ideal, the alternative is
    // either ignoring the print, a complete deadlock, or writing unsafely without locking.
    let mut stdout = STDOUT
        .try_lock()
        .expect("Attempted to print while stdout was already locked.");

    // Format the arguments into a byte buffer before printing them.
    // This lets us calculate if the bytes will overflow the buffer before printing them.
    let formatted_bytes = format!("{args}").into_bytes();
    let remaining_bytes_in_buffer = unsafe { vexSerialWriteFree(STDIO_CHANNEL) as usize };

    // Write all of our data in chunks the size of the outgoing serial buffer.
    // This ensures that writes of length greater than [`Stdout::INTERNAL_BUFFER_SIZE`] can still be
    // written by flushing several times.
    for chunk in formatted_bytes.chunks(Stdout::BUFFER_SIZE) {
        // If this chunk would overflow the buffer and cause a panic during `write_all`, wait for
        // the buffer to clear. Not only does this prevent a panic (if the panic handler prints it
        // could cause a recursive panic and immediately exit. **Very bad**), but it also allows
        // prints and device comms inside of tight loops that have a print.
        if remaining_bytes_in_buffer < chunk.len() {
            // Flushing is infallible, so we can unwrap here.
            stdout.flush().unwrap();
        }

        // Re-use the buffer to write the formatted bytes to the serial output. This technically
        // should never error because we have already flushed the buffer if it would overflow.
        if let Err(e) = stdout.write_all(chunk) {
            panic!("failed printing to stdout: {e}");
        }
    }
}

macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::serial::print_inner(format_args!($($arg)*));
    };
}

macro_rules! println {
    () => {
        $crate::serial::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::serial::print!("{}\n", format_args!($($arg)*));
    }
}

pub(crate) use print;
pub(crate) use println;
