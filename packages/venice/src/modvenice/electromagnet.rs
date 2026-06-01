use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::obj::{Obj, ObjBase, ObjType};
use vexide_devices::smart::electromagnet::Electromagnet;

use crate::{
    devices,
    modvenice::{Exception, units::time::TimeUnitObj},
    registry::SmartGuard,
};

/// An electromagnet plugged into a smart port.
///
/// The V5 electromagnet is a device unique to the V5 workcell kit. It is a simple device that
/// produces a magnetic field at a provided power level. The power level does not have specific
/// units.
///
/// # Hardware Overview
///
/// Not much information can be found on the V5 workcell electromagnet; however, the electromagnet
/// is intended to be used to pick up V5 Workcell colored disks. We can assume that the lower bound
/// on the electromagnet's strength is the weight of a V5 Workcell colored disk. Assuming that the
/// plastic part of the disk is made of ABS plastic and the metal part is solid iron, the
/// electromagnet can lift at least ≈0.24oz based off of the CAD model files for the V5 Workcell kit
/// provided by VEX.
#[class(qstr!(Electromagnet))]
pub struct ElectromagnetObj {
    base: ObjBase,
    guard: SmartGuard<Electromagnet>,
}

#[class_methods]
impl ElectromagnetObj {
    /// Maximum duration that the magnet can be powered for, in milliseconds.
    #[constant]
    const MAX_POWER_DURATION_MS: i32 = Electromagnet::MAX_POWER_DURATION.as_millis() as i32;

    /// Creates a new electromagnet.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// electromagnet = Electromagnet(1)
    ///
    /// # Use the electromagnet.
    /// electromagnet.set_power(1, Electromagnet.MAX_POWER_DURATION_MS, MILLIS)
    /// electromagnet.set_power(-0.2, 1000, MILLIS)
    /// ```
    #[make_new]
    #[stub(sig = "(self, port: int) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port_number = reader.next_positional()?;
        Ok(Self {
            base: ty.into(),
            guard: devices::lock_port(port_number, Electromagnet::new),
        })
    }

    /// Sets the power level of the magnet for a given duration.
    ///
    /// Power is expressed as a number from [-1.0, 1.0]. Larger power values will result in a
    /// stronger force of attraction from the magnet.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// electromagnet = Electromagnet(1)
    /// electromagnet.set_power(1, Electromagnet.MAX_POWER_DURATION_MS, MILLIS)
    /// ```
    #[method(ty = var_between(min = 3, max = 3))]
    #[stub(sig = "(self, power: float, duration: float, unit: TimeUnit) -> None")]
    fn set_power(args: &[Obj]) -> Result<(), Exception> {
        let mut reader = Args::new(3, 0, args).reader();
        let this = reader.next_positional::<&Self>()?;

        let power = reader.next_positional::<f32>()?;
        let duration = reader.next_positional()?;
        let time_unit = reader.next_positional::<&TimeUnitObj>()?;

        Ok(this
            .guard
            .borrow_mut()
            .set_power(power as f64, time_unit.unit().float_to_dur(duration))?)
    }

    /// Returns the user-set power level as a number from [-1.0, 1.0].
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// electromagnet = Electromagnet(1)
    /// electromagnet.set_power(0.5, Electromagnet.MAX_POWER_DURATION_MS, MILLIS)
    ///
    /// power = electromagnet.get_power()
    /// print(f"Power: {power:.1%}")
    /// ```
    #[method]
    fn get_power(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().power()? as f32)
    }

    /// Returns the magnet's electrical current in amps.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// electromagnet = Electromagnet(1)
    /// electromagnet.set_power(1, Electromagnet.MAX_POWER_DURATION_MS, MILLIS)
    ///
    /// current = electromagnet.get_current()
    /// print(f"Current: {current}A")
    /// ```
    #[method]
    fn get_current(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().current()? as f32)
    }

    /// Returns the internal temperature of the magnet in degrees celsius.
    ///
    /// # Errors
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// electromagnet = Electromagnet(1)
    ///
    /// temperature = electromagnet.get_temperature()
    /// print(f"Temperature: {temperature}°C")
    /// ```
    #[method]
    fn get_temperature(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().temperature()? as f32)
    }

    /// Returns the status code of the magnet.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// electromagnet = Electromagnet(1)
    ///
    /// status = electromagnet.get_status()
    /// print(f"Status: 0x{status:08x}")
    /// ```
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().status()? as i32)
    }

    /// Release this device and free its Smart Port lock. This binding will become unusable after
    /// this call, but you can reuse the underlying Smart Port number in a new device.
    ///
    /// Any attempts to use this device after freeing will result in a `ValueError` being raised.
    ///
    /// # Raises
    ///
    /// `ValueError`: If the device has already been freed.
    ///
    /// # Examples
    ///
    /// Construct a `Motor`, free it, then construct a `RotationSensor` with the same Smart Port:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.free()
    /// # `motor` is now unusable, but port 1 can be used in another device, such as a `RotationSensor`
    /// rotation_sensor = RotationSensor(1)
    /// ```
    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
