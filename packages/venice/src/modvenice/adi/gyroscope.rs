use std::cell::{Cell, RefCell};

use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::raise_stop_iteration,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};
use vex_sdk_jumptable::vexDeviceAdiValueSet;
use vexide_devices::adi::{
    AdiDevice,
    gyroscope::{AdiGyroscope, YawError},
};

use crate::modvenice::{
    Exception,
    adi::{adi_port_index, expander::AdiPortParser, validate_expander},
    device_error, device_handle,
    units::{rotation::RotationUnitObj, time::TimeUnitObj},
};

/// ADI Gyroscope
///
/// This class provides an interface for interacting with an ADI gyroscope device. The gyroscope
/// can be used to measure the yaw rotation of your robot.
///
/// # Hardware overview
///
/// The ADI gyroscope is a [LY3100ALH MEMS motion sensor](https://content.vexrobotics.com/docs/276-2333-Datasheet-1011.pdf).
/// This means that it can measure the rate of rotation up to ±1000 degrees per second.
/// VEXos only provides the calculated yaw rotation of the robot.
///
/// The gyroscope is rated for a noise density of 0.016 dps/√Hz (degrees per second per square root
/// of Hertz). This means that we cannot determine the exact amount of noise in the sensor's
/// readings because it is unknown how often VEXos polls the gyroscope.
///
/// An ADI gyroscope.
#[class(qstr!(AdiGyroscope))]
#[repr(C)]
pub struct AdiGyroscopeObj {
    base: ObjBase,
    gyro: RefCell<AdiGyroscope>,
}

#[derive(Debug, Clone, Copy)]
enum FutureState {
    /// Tell VEXos to start calibration for the given duration.
    Calibrate { calibration_duration_millis: i32 },
    /// Waiting for the calibration to start.
    WaitingStart,
    /// Waiting for the calibration to end.
    WaitingEnd,
}

/// An awaitable that calibrates an `AdiGyroscope` for a given duration.
///
/// Awaiting it starts calibration, cooperatively waits for VEXos to finish, and returns `None`. The
/// current calibration implementation selects a device handle from the ADI subport index instead of
/// the Brain or expander containing that port, so it cannot reliably target its sensor. Although
/// `AdiGyroscopeFuture` is root-importable, users normally obtain an instance from
/// `AdiGyroscope.calibrate` rather than constructing it directly.
///
/// # Raises
///
/// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
#[class(qstr!(AdiGyroscopeFuture))]
#[repr(C)]
pub struct AdiGyroscopeFuture {
    base: ObjBase,
    gyro: Obj,
    state: Cell<FutureState>,
}

impl From<YawError> for Exception {
    fn from(value: YawError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class_methods]
impl AdiGyroscopeObj {
    /// Creates a new gyroscope on the given `port`.
    ///
    /// `port` is an onboard ADI label from `"A"` through `"H"`, or an unused `AdiExpanderPort`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     gyro = AdiGyroscope("A")
    ///     # Do something with the gyroscope.
    ///     await gyro.calibrate(2, SECOND)
    ///     print(gyro.get_yaw(DEGREES))
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` is invalid or already occupied.
    #[make_new]
    #[stub(sig = "(self, port: str | AdiExpanderPort) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional_with(AdiPortParser)?;

        Ok(Self {
            base: ty.into(),
            gyro: RefCell::new(AdiGyroscope::new(port)),
        })
    }

    /// Returns `True` if the gyroscope is still calibrating.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// gyro = AdiGyroscope("A")
    /// print("Is calibrating:", gyro.is_calibrating())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the associated ADI expander is disconnected or is the wrong device type.
    #[method]
    fn is_calibrating(&self) -> Result<bool, Exception> {
        Ok(self.gyro.borrow().is_calibrating()?)
    }

    /// Calibrates the gyroscope for `duration` measured in `unit`.
    ///
    /// Keep the sensor stationary until the awaitable completes. `duration` should be finite and
    /// non-negative; the binding currently relies on the underlying duration conversion rather than
    /// raising a Python exception for invalid values. The awaitable also currently selects the hardware
    /// device handle from the ADI subport index, so calibration cannot reliably target this gyroscope
    /// until that implementation defect is corrected.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     gyro = AdiGyroscope("A")
    ///     print("Calibrating...")
    ///     await gyro.calibrate(2, SECOND)
    ///     print("Calibration completed successfully")
    ///
    /// vasyncio.run(main())
    /// ```
    #[method]
    fn calibrate(this: Obj, duration: f32, unit: &TimeUnitObj) -> AdiGyroscopeFuture {
        AdiGyroscopeFuture {
            base: AdiGyroscopeFuture::OBJ_TYPE.into(),
            gyro: this,
            state: Cell::new(FutureState::Calibrate {
                calibration_duration_millis: unit.unit().float_to_dur(duration).as_millis() as i32,
            }),
        }
    }

    /// Returns the measured yaw rotation of the gyroscope in the supplied `unit`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     gyro = AdiGyroscope("A")
    ///     await gyro.calibrate(2, SECOND)
    ///     print("Yaw:", gyro.get_yaw(DEGREES))
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If calibration is still running, the associated ADI expander is disconnected, or
    ///   it is the wrong device type.
    #[method]
    fn get_yaw(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        Ok(unit.unit().angle_to_float(self.gyro.borrow().yaw()?))
    }
}

#[class_methods]
impl AdiGyroscopeFuture {
    #[iter]
    extern "C" fn next(self_in: Obj) -> Obj {
        let this = self_in.try_as_obj::<Self>().unwrap();
        let gyro_obj = this.gyro.try_as_obj::<AdiGyroscopeObj>().unwrap();
        let gyro = gyro_obj.gyro.borrow();

        match this.state.get() {
            FutureState::Calibrate {
                calibration_duration_millis,
            } => match validate_expander(gyro.expander_port_number()) {
                Ok(()) => {
                    let port_number = gyro.port_numbers()[0];
                    let index = adi_port_index(port_number);
                    unsafe {
                        vexDeviceAdiValueSet(
                            device_handle(index),
                            index,
                            calibration_duration_millis,
                        );
                    }
                    this.state.set(FutureState::WaitingStart);
                    Obj::NONE
                }
                Err(error) => Exception::from(error).raise(token()),
            },
            FutureState::WaitingStart => match gyro.is_calibrating() {
                Ok(false) => Obj::NONE,
                Ok(true) => {
                    this.state.set(FutureState::WaitingEnd);
                    Obj::NONE
                }
                Err(error) => Exception::from(error).raise(token()),
            },
            FutureState::WaitingEnd => match gyro.is_calibrating() {
                Ok(false) => raise_stop_iteration(token(), Obj::NONE),
                Ok(true) => Obj::NONE,
                Err(error) => Exception::from(error).raise(token()),
            },
        }
    }
}
