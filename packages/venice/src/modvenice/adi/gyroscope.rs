use std::cell::{Cell, RefCell};

use argparse::{Args, error_msg};
use micropython_rs::{
    class, class_methods,
    except::raise_stop_iteration,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};
use vex_sdk_jumptable::vexDeviceAdiValueSet;
use vexide_devices::adi::{
    AdiDevice,
    gyroscope::{AdiGyroscope, YawError},
};

use crate::{
    devices,
    modvenice::{
        Exception,
        adi::{adi_port_index, validate_expander},
        device_error, device_handle,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
        vasyncio::event_loop::WAKE_SIGNAL,
    },
};

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
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional()?;

        Ok(Self {
            base: ty.into(),
            gyro: RefCell::new(AdiGyroscope::new(devices::lock_adi_port(port))),
        })
    }

    #[method]
    fn is_calibrating(&self) -> Result<bool, Exception> {
        Ok(self.gyro.borrow().is_calibrating()?)
    }

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
                    Obj::from_static(&WAKE_SIGNAL)
                }
                Err(error) => Exception::from(error).raise(token()),
            },
            FutureState::WaitingStart => match gyro.is_calibrating() {
                Ok(false) => Obj::from_static(&WAKE_SIGNAL),
                Ok(true) => {
                    this.state.set(FutureState::WaitingEnd);
                    Obj::from_static(&WAKE_SIGNAL)
                }
                Err(error) => Exception::from(error).raise(token()),
            },
            FutureState::WaitingEnd => match gyro.is_calibrating() {
                Ok(false) => raise_stop_iteration(token(), Obj::NONE),
                Ok(true) => Obj::from_static(&WAKE_SIGNAL),
                Err(error) => Exception::from(error).raise(token()),
            },
        }
    }
}
