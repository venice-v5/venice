use std::cell::Cell;

use micropython_rs::{
    const_dict,
    except::{raise_stop_iteration, raise_value_error},
    fun::Fun1,
    init::token,
    make_new_from_fn,
    obj::{Iter, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vex_sdk::{V5_DeviceT, vexDeviceGetByIndex, vexDeviceImuReset, vexDeviceImuStatusGet};
use vexide_devices::smart::{
    SmartDevice,
    imu::{InertialOrientation, InertialSensor, InertialStatus},
};

use crate::{
    args::Args,
    devices::{self, PortNumber},
    fun::{fun1, fun2, fun3},
    modvenice::{
        math::{EulerAngles, Quaternion, Vec3},
        raise_device_error, raise_port_error,
        units::{rotation::RotationUnitObj, time::TimeUnitObj},
        vasyncio::{event_loop::WAKE_SIGNAL, time32},
    },
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

#[repr(C)]
pub struct InertialSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, InertialSensor>,
}

#[repr(C)]
pub struct CalibrateFuture {
    base: ObjBase<'static>,
    state: Cell<CalibrateFutureState>,
    imu: Obj,
}

static IMU_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(InertialSensor))
            .set_make_new(make_new_from_fn!(imu_make_new))
            .set_locals_dict(const_dict![
                qstr!(CALIBRATION_START_TIMEOUT_MS) => Obj::from_int(InertialSensor::CALIBRATION_START_TIMEOUT.as_millis() as i32),
                qstr!(CALIBRATION_END_TIMEOUT_MS) => Obj::from_int(InertialSensor::CALIBRATION_END_TIMEOUT.as_millis() as i32),
                qstr!(MIN_DATA_INTERVAL_MS) => Obj::from_int(InertialSensor::MIN_DATA_INTERVAL.as_millis() as i32),

                qstr!(calibrate) => Obj::from_static(&Fun1::new(imu_calibrate)),
                qstr!(get_heading) => Obj::from_static(&fun2!(imu_get_heading, &InertialSensorObj, &RotationUnitObj)),
                qstr!(set_heading) => Obj::from_static(&fun3!(imu_set_heading, &InertialSensorObj, f32, &RotationUnitObj)),
                qstr!(reset_heading) => Obj::from_static(&fun1!(imu_reset_heading, &InertialSensorObj)),

                qstr!(get_rotation) => Obj::from_static(&fun2!(imu_get_rotation, &InertialSensorObj, &RotationUnitObj)),
                qstr!(set_rotation) => Obj::from_static(&fun3!(imu_set_rotation, &InertialSensorObj, f32, &RotationUnitObj)),
                qstr!(reset_rotation) => Obj::from_static(&fun1!(imu_reset_rotation, &InertialSensorObj)),

                qstr!(get_euler) => Obj::from_static(&fun2!(imu_get_euler, &InertialSensorObj, &RotationUnitObj)),
                qstr!(get_quaternion) => Obj::from_static(&fun1!(imu_get_quaternion, &InertialSensorObj)),
                qstr!(get_gyro_rate) => Obj::from_static(&fun1!(imu_get_gyro_rate, &InertialSensorObj)),
                qstr!(get_acceleration) => Obj::from_static(&fun1!(imu_get_acceleration, &InertialSensorObj)),

                //qstr!(get_status) => Obj::from_static(&fun1!(imu_get_status, &InertialSensorObj)),
                qstr!(is_calibrating) => Obj::from_static(&fun1!(imu_is_calibrating, &InertialSensorObj)),
                qstr!(is_auto_calibrated) => Obj::from_static(&fun1!(imu_is_auto_calibrated, &InertialSensorObj)),
                qstr!(get_physical_orientation) => Obj::from_static(&fun1!(imu_get_physical_orientation, &InertialSensorObj)),
                qstr!(set_data_interval) => Obj::from_static(&fun3!(imu_set_data_interval, &InertialSensorObj, f32, &TimeUnitObj)),
            ]);

static CALIBRATE_FUTURE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(CalibrateFuture))
        .set_iter(Iter::IterNext(calibrate_future_iternext));

unsafe impl ObjTrait for InertialSensorObj {
    const OBJ_TYPE: &ObjType = IMU_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for CalibrateFuture {
    const OBJ_TYPE: &ObjType = CALIBRATE_FUTURE_OBJ_TYPE.as_obj_type();
}

fn imu_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let mut reader = Args::new(n_pos, n_kw, args).reader(token());
    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token(), c"port number must be between 1 and 21"));

    let guard = devices::lock_port(port, InertialSensor::new);

    alloc_obj(InertialSensorObj {
        base: ObjBase::new(ty),
        guard,
    })
}

extern "C" fn imu_calibrate(self_in: Obj) -> Obj {
    alloc_obj(CalibrateFuture {
        base: ObjBase::new(CalibrateFuture::OBJ_TYPE),
        imu: self_in,
        state: Cell::new(CalibrateFutureState::Calibrate),
    })
}

fn imu_get_heading(this: &InertialSensorObj, unit: &RotationUnitObj) -> Obj {
    Obj::from_float(
        unit.unit().angle_to_float(
            this.guard
                .borrow()
                .heading()
                .unwrap_or_else(|e| raise_port_error!(e)),
        ),
    )
}

fn imu_set_heading(this: &InertialSensorObj, heading: f32, unit: &RotationUnitObj) -> Obj {
    let angle = unit.unit().float_to_angle(heading);
    this.guard
        .borrow_mut()
        .set_heading(angle)
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
}

fn imu_reset_heading(this: &InertialSensorObj) -> Obj {
    this.guard
        .borrow_mut()
        .reset_heading()
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
}

fn imu_get_rotation(this: &InertialSensorObj, unit: &RotationUnitObj) -> Obj {
    Obj::from_float(
        unit.unit().angle_to_float(
            this.guard
                .borrow()
                .heading()
                .unwrap_or_else(|e| raise_port_error!(e)),
        ),
    )
}

fn imu_set_rotation(this: &InertialSensorObj, rotation: f32, unit: &RotationUnitObj) -> Obj {
    let angle = unit.unit().float_to_angle(rotation);
    this.guard
        .borrow_mut()
        .set_rotation(angle)
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
}

fn imu_reset_rotation(this: &InertialSensorObj) -> Obj {
    this.guard
        .borrow_mut()
        .reset_rotation()
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
}

fn imu_get_euler(this: &InertialSensorObj, unit: &RotationUnitObj) -> Obj {
    alloc_obj(EulerAngles::new(
        this.guard
            .borrow()
            .euler()
            .unwrap_or_else(|e| raise_port_error!(e)),
        unit.unit(),
    ))
}

fn imu_get_quaternion(this: &InertialSensorObj) -> Obj {
    alloc_obj(Quaternion::new(
        this.guard
            .borrow()
            .quaternion()
            .unwrap_or_else(|e| raise_port_error!(e)),
    ))
}

fn imu_get_gyro_rate(this: &InertialSensorObj) -> Obj {
    alloc_obj(Vec3::new(
        this.guard
            .borrow()
            .gyro_rate()
            .unwrap_or_else(|e| raise_port_error!(e)),
    ))
}

fn imu_get_acceleration(this: &InertialSensorObj) -> Obj {
    alloc_obj(Vec3::new(
        this.guard
            .borrow()
            .acceleration()
            .unwrap_or_else(|e| raise_port_error!(e)),
    ))
}

// TODO: figure out how to return the bitflags struct InertialStatus
// fn imu_get_status(this: &InertialSensorObj) -> Obj {
//     todo!()
// }

fn imu_is_calibrating(this: &InertialSensorObj) -> Obj {
    Obj::from_bool(
        this.guard
            .borrow()
            .is_calibrating()
            .unwrap_or_else(|e| raise_port_error!(e)),
    )
}

fn imu_is_auto_calibrated(this: &InertialSensorObj) -> Obj {
    Obj::from_bool(
        this.guard
            .borrow()
            .is_auto_calibrated()
            .unwrap_or_else(|e| raise_port_error!(e)),
    )
}

#[repr(C)]
pub struct InertialOrientationObj {
    base: ObjBase<'static>,
    orientation: InertialOrientation,
}

impl InertialOrientationObj {
    const fn new(orientation: InertialOrientation) -> Self {
        Self {
            base: ObjBase::new(INERTIAL_ORIENTATION_OBJ.as_obj_type()),
            orientation,
        }
    }
}

mod orientations {
    use super::*;

    pub static X_DOWN: InertialOrientationObj =
        InertialOrientationObj::new(InertialOrientation::XDown);
    pub static X_UP: InertialOrientationObj = InertialOrientationObj::new(InertialOrientation::XUp);

    pub static Y_DOWN: InertialOrientationObj =
        InertialOrientationObj::new(InertialOrientation::YDown);
    pub static Y_UP: InertialOrientationObj = InertialOrientationObj::new(InertialOrientation::YUp);

    pub static Z_DOWN: InertialOrientationObj =
        InertialOrientationObj::new(InertialOrientation::ZDown);
    pub static Z_UP: InertialOrientationObj = InertialOrientationObj::new(InertialOrientation::ZUp);
}

pub static INERTIAL_ORIENTATION_OBJ: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(InertialOrientation)).set_locals_dict(const_dict![
        qstr!(X_DOWN) => Obj::from_static(&orientations::X_DOWN),
        qstr!(X_UP) => Obj::from_static(&orientations::X_UP),

        qstr!(Y_DOWN) => Obj::from_static(&orientations::Y_DOWN),
        qstr!(Y_UP) => Obj::from_static(&orientations::Y_UP),

        qstr!(Z_DOWN) => Obj::from_static(&orientations::Z_DOWN),
        qstr!(Z_UP) => Obj::from_static(&orientations::Z_UP),
    ]);

unsafe impl ObjTrait for InertialOrientationObj {
    const OBJ_TYPE: &ObjType = INERTIAL_ORIENTATION_OBJ.as_obj_type();
}

fn imu_get_physical_orientation(this: &InertialSensorObj) -> Obj {
    match this
        .guard
        .borrow()
        .physical_orientation()
        .unwrap_or_else(|e| raise_port_error!(e))
    {
        InertialOrientation::XDown => Obj::from_static(&orientations::X_DOWN),
        InertialOrientation::XUp => Obj::from_static(&orientations::X_UP),

        InertialOrientation::YDown => Obj::from_static(&orientations::Y_DOWN),
        InertialOrientation::YUp => Obj::from_static(&orientations::Y_UP),

        InertialOrientation::ZDown => Obj::from_static(&orientations::Z_DOWN),
        InertialOrientation::ZUp => Obj::from_static(&orientations::Z_UP),
    }
}

fn imu_set_data_interval(this: &InertialSensorObj, interval: f32, unit: &TimeUnitObj) -> Obj {
    if interval < 0.0 {
        raise_value_error(token(), c"interval cannot be negative");
    }
    let dur = unit.unit().float_to_dur(interval);
    this.guard
        .borrow_mut()
        .set_data_interval(dur)
        .unwrap_or_else(|e| raise_port_error!(e));
    Obj::NONE
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

extern "C" fn calibrate_future_iternext(self_in: Obj) -> Obj {
    let this = self_in.try_as_obj::<CalibrateFuture>().unwrap();

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
        // This only happens for one poll of the future (the first one). All future polls will
        // either be waiting for calibration to start or for calibration to end.
        CalibrateFutureState::Calibrate => {
            // Check if the sensor was already calibrating before we recalibrate it ourselves.
            //
            // This can happen at the start of program execution or if the sensor loses then
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

        // In this stage, we are either waiting for the calibration status flag to be set
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

            if status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::Start {
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
