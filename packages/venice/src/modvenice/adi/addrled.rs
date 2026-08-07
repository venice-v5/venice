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

/// ADI Addressable LEDs
///
/// This class provides an interface for controlling [WS2812B] addressable LED strips over ADI
/// ports. These are commonly used for decorative lighting. More can be read about using them in
/// [this blog post](https://sylvie.fyi/posts/v5-addrled/) and
/// [this forum post](https://www.vexforum.com/t/v5-addressable-leds/106960).
///
/// # Hardware Overview
///
/// ADI ports are capable of controlling a WS2812B LED strip with up to 64 diodes per set of 8 ADI
/// ports. This limitation is due to the 2A current limit on ADI ports — plugging multiple strips
/// into the same set of ADI ports may cause your lights to flicker due to this limit being reached.
/// If you require more than 64 continuously running diodes, then you can run each strip through its
/// own `AdiExpander`.
///
/// The V5's ADI ports can present some technical challenges when interfacing with LEDs. Some
/// commercially available strips will not work with the V5 out of the box, but mileage may vary.
/// This is mainly caused by two "quirks" of the V5's ADI ports:
///
/// - ADI ports operate at 3.3V digital logic, but most WS2812B strips expect 5V logic.
/// - The Brain's ADI ports include built-in short protection via a 1kΩ resistor that may impact
///   signal timing on some strips, slowing down the edges of digital logic pulses sent to strip. In
///   rare cases, this can cause issues with some strips.
///
/// Using something like a [74HCT125 buffer] inline with the output to convert the 3.3-5V logic
/// addresses both these problems.
///
/// [WS2812B]: https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf
/// [74HCT125 buffer]: https://www.diodes.com/assets/Datasheets/74HCT125.pdf
///
/// WS2812B Addressable LED Strip
#[class(qstr!(AdiAddrLed))]
pub struct AdiAddrLedObj {
    base: ObjBase,
    led: RefCell<AdiAddrLedVar>,
}

#[class_methods]
impl AdiAddrLedObj {
    /// Initializes an LED strip with a given `count` on an ADI port.
    ///
    /// `count` must be from 0 through 64. `port` is an onboard ADI label from `"A"` through `"H"`, or an
    /// unused `AdiExpanderPort`. The current binding incorrectly accepts exactly one positional argument
    /// before attempting to read both arguments, so construction is not reachable.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new LED strip with 8 addressable pixels.
    /// leds = AdiAddrLed("A", 8)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: Because the binding's argument-count validation does not match this signature.
    /// - `ValueError`: If `port` is invalid or occupied, or `count` is outside 0 through 64.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort, count: int) -> None")]
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

    /// Attempts to write a buffer of colors to the LED strip. Returns how many colors were actually
    /// written.
    ///
    /// `buffer` must expose contiguous unsigned 32-bit packed colors. At most the configured pixel count
    /// is reported, although the binding passes the entire buffer to the device.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    /// from array import array
    ///
    /// # Create a new LED strip with 8 addressable pixels.
    /// leds = AdiAddrLed("A", 8)
    ///
    /// # List of colors that each LED pixel will be set to.
    /// colors = array("L", [
    ///     Color.RED.as_int(),
    ///     Color.YELLOW.as_int(),
    ///     Color.GREEN.as_int(),
    ///     Color.BLUE.as_int(),
    ///     Color.PURPLE.as_int(),
    ///     Color.RED.as_int(),
    ///     Color.YELLOW.as_int(),
    ///     Color.GREEN.as_int(),
    /// ])
    ///
    /// leds.set_buffer(colors)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `buffer` does not provide a compatible buffer.
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    #[stub(sig = "(self, buffer: Any) -> int")]
    fn set_buffer(&self, buf: Buffer<'_, u32>) -> Result<i32, Exception> {
        Ok(self.led.borrow_mut().set_buffer(buf.buffer())? as i32)
    }

    /// Sets the color of an individual diode on the strip.
    ///
    /// Valid `index` values are intended to run from zero through one less than the configured count. The
    /// current range check also accepts an index equal to the count, which addresses past the strip.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new LED strip with 8 addressable pixels.
    /// leds = AdiAddrLed("A", 8)
    ///
    /// # Set the first pixel in the strip to white.
    /// leds.set_pixel(0, Color.WHITE)
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `index` is greater than the configured pixel count.
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
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

    /// Sets the entire LED strip to one color.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a new LED strip with 8 addressable pixels.
    /// leds = AdiAddrLed("A", 8)
    ///
    /// # Set all pixels to white.
    /// leds.set_all(Color.WHITE)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn set_all(&self, color: &ColorObj) -> Result<(), Exception> {
        Ok(self.led.borrow_mut().set_all(color.color().into_raw())?)
    }
}
