use std::cell::Cell;

use argparse::Args;
use micropython_rs::{
    class, class_methods,
    except::{raise_stop_iteration, raise_value_error},
    fun::Fun1,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};
use vex_sdk::{V5_DeviceT, vexDeviceGetByIndex, vexDeviceImuReset, vexDeviceImuStatusGet};
use vexide_devices::smart::{
    SmartDevice,
    imu::{InertialOrientation, InertialSensor, InertialStatus},
};

use crate::{
    devices::{self, PortNumber},
    modvenice::{
        math::{EulerAngles, Quaternion, Vec3},
        raise_device_error, raise_port_error,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
        vasyncio::{event_loop::WAKE_SIGNAL, time32},
    },
    obj::alloc_obj,
    registry::RegistryGuard,
};

#[class(qstr!(InertialSensor))]
#[repr(C)]
pub struct InertialSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, InertialSensor>,
}

#[class(qstr!(CalibrateFuture))]
#[repr(C)]
pub struct CalibrateFuture {
    base: ObjBase<'static>,
    state: Cell<CalibrateFutureState>,
    imu: Obj,
}

#[class(qstr!(InertialOrientation))]
#[repr(C)]
pub struct InertialOrientationObj {
    base: ObjBase<'static>,
    orientation: InertialOrientation,
}

#[class_methods]
impl InertialOrientationObj {
    const fn new(orientation: InertialOrientation) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            orientation,
        }
    }

    #[constant]
    pub const X_DOWN: &Self = &Self::new(InertialOrientation::XDown);
    #[constant]
    pub const X_UP: &Self = &Self::new(InertialOrientation::XUp);
    #[constant]
    pub const Y_DOWN: &Self = &Self::new(InertialOrientation::YDown);
    #[constant]
    pub const Y_UP: &Self = &Self::new(InertialOrientation::YUp);
    #[constant]
    pub const Z_DOWN: &Self = &Self::new(InertialOrientation::ZDown);
    #[constant]
    pub const Z_UP: &Self = &Self::new(InertialOrientation::ZUp);
}

#[class_methods]
impl InertialSensorObj {
    #[make_new]
    fn make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
        let mut reader = Args::new(n_pos, n_kw, args).reader(token());
        let port = PortNumber::from_i32(reader.next_positional()).unwrap_or_else(|_| {
            raise_value_error(token(), c"port number must be between 1 and 21")
        });

        let guard = devices::lock_port(port, InertialSensor::new);

        alloc_obj(InertialSensorObj {
            base: ObjBase::new(ty),
            guard,
        })
    }

    extern "C" fn calibrate(self_in: Obj) -> Obj {
        alloc_obj(CalibrateFuture {
            base: ObjBase::new(CalibrateFuture::OBJ_TYPE),
            imu: self_in,
            state: Cell::new(CalibrateFutureState::Calibrate),
        })
    }

    #[constant(qstr!(calibrate))]
    const CALIBRATE: &Fun1 = &Fun1::new(Self::calibrate);

    #[method]
    fn get_heading(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit().angle_to_float(
            self.guard
                .borrow()
                .heading()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn set_heading(&self, heading: f32, unit: &RotationUnitObj) {
        let angle = unit.unit().float_to_angle(heading);
        self.guard
            .borrow_mut()
            .set_heading(angle)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn reset_heading(&self) {
        self.guard
            .borrow_mut()
            .reset_heading()
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_rotation(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit().angle_to_float(
            self.guard
                .borrow()
                .heading()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn set_rotation(&self, rotation: f32, unit: &RotationUnitObj) {
        let angle = unit.unit().float_to_angle(rotation);
        self.guard
            .borrow_mut()
            .set_rotation(angle)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn reset_rotation(&self) {
        self.guard
            .borrow_mut()
            .reset_rotation()
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_euler(&self, unit: &RotationUnitObj) -> EulerAngles {
        EulerAngles::new(
            self.guard
                .borrow()
                .euler()
                .unwrap_or_else(|e| raise_port_error!(e)),
            unit.unit(),
        )
    }

    #[method]
    fn get_quaternion(&self) -> Quaternion {
        Quaternion::new(
            self.guard
                .borrow()
                .quaternion()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn get_gyro_rate(&self) -> Vec3 {
        Vec3::new(
            self.guard
                .borrow()
                .gyro_rate()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn get_acceleration(&self) -> Vec3 {
        Vec3::new(
            self.guard
                .borrow()
                .acceleration()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    // TODO: figure out how to return the bitflags struct InertialStatus
    // fn get_status(&self) -> _ {
    //     todo!()
    // }

    #[method]
    fn is_calibrating(&self) -> bool {
        self.guard
            .borrow()
            .is_calibrating()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn is_auto_calibrated(&self) -> bool {
        self.guard
            .borrow()
            .is_auto_calibrated()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn get_physical_orientation(&self) -> Obj {
        match self
            .guard
            .borrow()
            .physical_orientation()
            .unwrap_or_else(|e| raise_port_error!(e))
        {
            InertialOrientation::XDown => Obj::from_static(InertialOrientationObj::X_DOWN),
            InertialOrientation::XUp => Obj::from_static(InertialOrientationObj::X_UP),

            InertialOrientation::YDown => Obj::from_static(InertialOrientationObj::Y_DOWN),
            InertialOrientation::YUp => Obj::from_static(InertialOrientationObj::Y_UP),

            InertialOrientation::ZDown => Obj::from_static(InertialOrientationObj::Z_DOWN),
            InertialOrientation::ZUp => Obj::from_static(InertialOrientationObj::Z_UP),
        }
    }

    #[method]
    fn set_data_interval(&self, interval: f32, unit: &TimeUnitObj) {
        if interval < 0.0 {
            raise_value_error(token(), c"interval cannot be negative");
        }
        let dur = unit.unit().float_to_dur(interval);
        self.guard
            .borrow_mut()
            .set_data_interval(dur)
            .unwrap_or_else(|e| raise_port_error!(e));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CalibrationPhase {
    Status,
    Start,
    End,
}

#[derive(Clone, Copy)]
pub enum CalibrateFutureState {
    Calibrate,
    Waiting {
        timestamp: time32::Instant,
        phase: CalibrationPhase,
    },
}

fn smart_port_index(n: u8) -> u32 {
    (n - 1) as u32
}

unsafe fn device_handle(index: u32) -> V5_DeviceT {
    unsafe { vexDeviceGetByIndex(index) }
}

#[class_methods]
impl CalibrateFuture {
    #[iter]
    extern "C" fn iter(self_in: Obj) -> Obj {
        let this = self_in.try_as_obj::<Self>().unwrap();

        let imu = this
            .imu
            .try_as_obj::<InertialSensorObj>()
            .unwrap()
            .guard
            .borrow();

        let device = unsafe { device_handle(smart_port_index(imu.port_number())) };

        // Get the sensor's status flags, which tell us whether or not we are still calibrating.
        let status = InertialStatus::from_bits_retain(if let Err(e) = imu.validate_port() {
            // IMU got unplugged, so we'll resolve early.
            raise_port_error!(e);
        } else {
            // Get status flags from VEXos.
            let flags = unsafe { vexDeviceImuStatusGet(device) };
            if flags == 0x0 {
                this.state.set(CalibrateFutureState::Waiting {
                    timestamp: time32::Instant::now(),
                    phase: CalibrationPhase::Status,
                });
            }

            flags
        });

        match this.state.get() {
            // The "calibrate" phase begins the calibration process.
            //
            // self only happens for one poll of the future (the first one). All future polls will
            // either be waiting for calibration to start or for calibration to end.
            CalibrateFutureState::Calibrate => {
                // Check if the sensor was already calibrating before we recalibrate it ourselves.
                //
                // self can happen at the start of program execution or if the sensor loses then
                // regains power. In those instances, VEXos will automatically start
                // the calibration process without us asking.
                // Calling [`vexDeviceImuReset`] while calibration is already happening has caused
                // bugs in our testing, so we instead just want to wait until the
                // calibration attempt has finished.
                //
                // See <https://github.com/vexide/vexide/issues/253> for more details.
                if status.contains(InertialStatus::CALIBRATING) {
                    // Sensor was already calibrating, so wait for that to finish.
                    this.state.set(CalibrateFutureState::Waiting {
                        timestamp: time32::Instant::now(),
                        phase: CalibrationPhase::End,
                    });
                } else {
                    // Request that VEXos calibrate the IMU, and transition to pending state.
                    unsafe { vexDeviceImuReset(device) }

                    // Change to waiting for calibration to start.
                    this.state.set(CalibrateFutureState::Waiting {
                        timestamp: time32::Instant::now(),
                        phase: CalibrationPhase::Start,
                    });
                }

                Obj::from_static(&WAKE_SIGNAL)
            }

            // In self stage, we are either waiting for the calibration status flag to be set
            // (CalibrationPhase::Start), indicating that calibration has begun, or we
            // are waiting for the calibration status flag to be cleared, indicating
            // that calibration has finished (CalibrationFlag::End).
            CalibrateFutureState::Waiting {
                timestamp: since,
                phase,
            } => {
                let elapsed = time32::Instant::now() - since;

                if elapsed
                    > time32::Duration::from_duration(match phase {
                        CalibrationPhase::Start | CalibrationPhase::Status => {
                            InertialSensor::CALIBRATION_START_TIMEOUT
                        }
                        CalibrationPhase::End => InertialSensor::CALIBRATION_END_TIMEOUT,
                    })
                {
                    // Waiting took too long and exceeded a timeout.
                    raise_device_error(token(), c"calibration timed out");
                }

                if status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::Start
                {
                    // We are in the "start" phase (waiting for the flag to be set) and the flag is
                    // now set, meaning that calibration has begun.
                    //
                    // We now know that the sensor is actually calibrating, so we transition to
                    // [`CalibrationPhase::End`] and reset the timeout timestamp to wait for
                    // calibration to finish.
                    this.state.set(CalibrateFutureState::Waiting {
                        timestamp: time32::Instant::now(),
                        phase: CalibrationPhase::End,
                    });
                } else if !status.is_empty() && phase == CalibrationPhase::Status {
                    this.state.set(CalibrateFutureState::Calibrate);
                } else if !status.contains(InertialStatus::CALIBRATING)
                    && phase == CalibrationPhase::End
                {
                    // The [`InertialStatus::CALIBRATING`] has been cleared, indicating that
                    // calibration is complete.
                    raise_stop_iteration(token(), Obj::NONE);
                }

                Obj::from_static(&WAKE_SIGNAL)
            }
        }
    }
}
