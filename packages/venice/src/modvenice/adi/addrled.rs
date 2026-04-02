use std::cell::RefCell;

use argparse::{Args, IntParser, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    buffer::Buffer,
    except::value_error,
    obj::{Obj, ObjBase, ObjType},
};
use vex_sdk_jumptable::vexDeviceAdiAddrLedSet;
use vexide_devices::{
    adi::{AdiDeviceType, AdiPort},
    smart::PortError,
};

use crate::modvenice::{
    Exception,
    adi::{
        adi_port_index, configure_port, device_handle, expander::AdiPortParser, validate_expander,
    },
    color::ColorObj,
};

struct AdiAddrLedVar {
    port: AdiPort,
    n: usize,
}

impl AdiAddrLedVar {
    fn new(port: AdiPort, n: usize) -> Self {
        // bounds checking on `n` is left to the object constructor
        configure_port(&port, AdiDeviceType::DigitalOut);
        Self { port, n }
    }

    // internal, do not expose to Python
    fn update(&mut self, buf: &[u32], offset: usize) {
        unsafe {
            vexDeviceAdiAddrLedSet(
                device_handle(&self.port),
                adi_port_index(self.port.number()),
                buf.as_ptr().cast_mut(),
                offset as u32,
                buf.len() as u32,
                0,
            );
        }
    }

    fn set_buffer(&mut self, buf: &[u32]) -> Result<usize, PortError> {
        validate_expander(self.port.expander_number())?;
        self.update(buf, 0);
        Ok(buf.len().min(self.n))
    }

    fn set_pixel(&mut self, index: usize, color: u32) -> Result<(), PortError> {
        // bounds checking on `index` is left to the Python method
        validate_expander(self.port.expander_number())?;
        self.update(&[color], index);
        Ok(())
    }

    fn set_all(&mut self, color: u32) -> Result<(), PortError> {
        validate_expander(self.port.expander_number())?;
        self.update(&vec![color; self.n], 0);
        Ok(())
    }
}

#[class(qstr!(AdiAddrLed))]
pub struct AdiAddrLedObj {
    base: ObjBase,
    led: RefCell<AdiAddrLedVar>,
}

#[class_methods]
impl AdiAddrLedObj {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;
        let n = reader.next_positional_with(IntParser::new(0..=64))?;

        Ok(Self {
            base: ty.into(),
            led: AdiAddrLedVar::new(port, n).into(),
        })
    }

    #[method]
    fn set_buffer(&self, buf: Buffer<'_, u32>) -> Result<i32, Exception> {
        Ok(self.led.borrow_mut().set_buffer(buf.buffer())? as i32)
    }

    #[method]
    fn set_pixel(&self, index: i32, color: &ColorObj) -> Result<(), Exception> {
        let mut led = self.led.borrow_mut();
        if index as usize > led.n {
            Err(value_error(error_msg!(
                "pixel index ({index}) is out of range for LED stripe size ({})",
                led.n,
            )))?
        }
        Ok(led.set_pixel(index as usize, color.color().into_raw())?)
    }

    #[method]
    fn set_all(&self, color: &ColorObj) -> Result<(), Exception> {
        Ok(self.led.borrow_mut().set_all(color.color().into_raw())?)
    }
}
