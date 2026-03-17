pub mod brake;
pub mod direction;
pub mod gearset;
pub mod motor_type;

use argparse::Args;
use brake::BrakeModeObj;
use direction::DirectionObj;
use gearset::GearsetObj;
use micropython_rs::{
    class, class_methods,
    except::type_error,
    init::token,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
};
use vexide_devices::{
    math::Direction,
    smart::motor::{Gearset, Motor},
};

use super::raise_device_error;
use crate::{
    devices::{self},
    modvenice::{
        Exception, motor::motor_type::MotorTypeObj, raise_port_error,
        units::rotation::RotationUnitObj,
    },
    registry::RegistryGuard,
};

#[class(qstr!(Motor))]
#[repr(C)]
pub struct MotorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, Motor>,
}

#[class_methods]
impl MotorObj {
    #[method(ty = var_between(min = 1, max = 3), binding = "static")]
    fn new_v5(args: &[Obj]) -> Result<Self, Exception> {
        let token = token();
        let mut reader = Args::new(args.len(), 0, args).reader(token);

        let port = reader.next_positional()?;
        let direction = reader.next_positional_or(DirectionObj::FORWARD)?;
        let gearset = reader.next_positional_or(GearsetObj::GREEN)?;

        let guard = devices::lock_port(port, |port| {
            Motor::new(port, gearset.gearset(), direction.direction())
        });

        if guard.borrow().is_exp() {
            guard.free().unwrap();
            raise_device_error(token, c"invalid motor type, expected V5, found Exp")
        }

        Ok(Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            guard,
        })
    }

    #[method(ty = var_between(min = 1, max = 2), binding = "static")]
    fn new_exp(args: &[Obj]) -> Result<Self, Exception> {
        let token = token();
        let mut reader = Args::new(args.len(), 0, args).reader(token);
        reader.assert_npos(1, 2).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let direction = reader.next_positional_or(DirectionObj::FORWARD)?;

        let guard = devices::lock_port(port, |port| Motor::new_exp(port, direction.direction()));
        if guard.borrow().is_v5() {
            guard.free().unwrap();
            raise_device_error(token, c"invalid motor type, expected Exp, found V5");
        }

        Ok(MotorObj {
            base: ObjBase::new(Self::OBJ_TYPE),
            guard,
        })
    }

    #[make_new]
    fn make_new(_: &ObjType, _: usize, n_kw: usize, args: &[Obj]) -> Result<Self, Exception> {
        if n_kw != 0 {
            Err(type_error(c"function does not accept keyword arguments").into())
        } else {
            Self::new_v5(args)
        }
    }

    #[method]
    fn set_voltage(&self, volts: f32) {
        self.guard
            .borrow_mut()
            .set_voltage(volts as f64)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_velocity(&self, rpm: i32) {
        self.guard
            .borrow_mut()
            .set_velocity(rpm)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn brake(&self, mode: &BrakeModeObj) {
        self.guard
            .borrow_mut()
            .brake(mode.mode())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_gearset(&self, gearset: &GearsetObj) {
        self.guard
            .borrow_mut()
            .set_gearset(gearset.gearset())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn is_exp(&self) -> bool {
        self.guard.borrow().is_exp()
    }

    #[method]
    fn is_v5(&self) -> bool {
        self.guard.borrow().is_v5()
    }

    #[method]
    fn get_max_voltage(&self) -> f32 {
        self.guard.borrow().max_voltage() as f32
    }

    #[method]
    fn gearset(&self) -> Obj {
        let gearset = self
            .guard
            .borrow()
            .gearset()
            .unwrap_or_else(|e| raise_port_error!(e));
        Obj::from_static(match gearset {
            Gearset::Red => GearsetObj::RED,
            Gearset::Green => GearsetObj::GREEN,
            Gearset::Blue => GearsetObj::BLUE,
        })
    }

    #[method(ty = var_between(min = 4, max = 4))]
    fn set_position_target(args: &[Obj]) -> Result<(), Exception> {
        let mut reader = Args::new(args.len(), 0, args).reader(token());

        let motor = reader.next_positional::<&MotorObj>().unwrap();
        let position_val = reader.next_positional()?;
        let unit_obj = reader.next_positional::<&RotationUnitObj>()?;
        let velocity_val = reader.next_positional()?;

        let angle = unit_obj.unit().float_to_angle(position_val);
        motor
            .guard
            .borrow_mut()
            .set_position_target(angle, velocity_val)
            .unwrap_or_else(|e| raise_port_error!(e));
        Ok(())
    }

    #[method]
    fn get_velocity(&self) -> f32 {
        self.guard
            .borrow()
            .velocity()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_power(&self) -> f32 {
        self.guard
            .borrow()
            .power()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_torque(&self) -> f32 {
        self.guard
            .borrow()
            .torque()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_voltage(&self) -> f32 {
        self.guard
            .borrow()
            .voltage()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_raw_position(&self) -> i32 {
        self.guard
            .borrow()
            .raw_position()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn get_current(&self) -> f32 {
        self.guard
            .borrow()
            .current()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_efficiency(&self) -> f32 {
        self.guard
            .borrow()
            .efficiency()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_current_limit(&self) -> f32 {
        self.guard
            .borrow()
            .current_limit()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_voltage_limit(&self) -> f32 {
        self.guard
            .borrow()
            .voltage_limit()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_temperature(&self) -> f32 {
        self.guard
            .borrow()
            .temperature()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn set_profiled_velocity(&self, velocity: i32) {
        self.guard
            .borrow_mut()
            .set_profiled_velocity(velocity)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn reset_position(&self) {
        self.guard
            .borrow_mut()
            .reset_position()
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_current_limit(&self, limit: f32) {
        self.guard
            .borrow_mut()
            .set_current_limit(limit as f64)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_voltage_limit(&self, limit: f32) {
        self.guard
            .borrow_mut()
            .set_voltage_limit(limit as f64)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn is_over_temperature(&self) -> bool {
        self.guard
            .borrow()
            .is_over_temperature()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn is_over_current(&self) -> bool {
        self.guard
            .borrow()
            .is_over_current()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn is_driver_fault(&self) -> bool {
        self.guard
            .borrow()
            .is_driver_fault()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn is_driver_over_current(&self) -> bool {
        self.guard
            .borrow()
            .is_driver_over_current()
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    #[method]
    fn get_motor_type(&self) -> Obj {
        let mt = self.guard.borrow().motor_type();
        Obj::from_static(MotorTypeObj::new_static(mt))
    }

    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> f32 {
        let angle = self
            .guard
            .borrow()
            .position()
            .unwrap_or_else(|e| raise_port_error!(e));
        unit.unit().angle_to_float(angle)
    }

    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) {
        let angle = unit.unit().float_to_angle(position);
        self.guard
            .borrow_mut()
            .set_position(angle)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_direction(&self, direction: &DirectionObj) {
        self.guard
            .borrow_mut()
            .set_direction(direction.direction())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_direction(&self) -> Obj {
        let dir = self
            .guard
            .borrow()
            .direction()
            .unwrap_or_else(|e| raise_port_error!(e));
        Obj::from_static(match dir {
            Direction::Forward => DirectionObj::FORWARD,
            Direction::Reverse => DirectionObj::REVERSE,
        })
    }

    #[method]
    fn get_status(&self) -> i32 {
        let status = self
            .guard
            .borrow()
            .status()
            .unwrap_or_else(|e| raise_port_error!(e));
        status.bits() as i32
    }

    #[method]
    fn get_faults(&self) -> i32 {
        let faults = self
            .guard
            .borrow()
            .faults()
            .unwrap_or_else(|e| raise_port_error!(e));
        faults.bits() as i32
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
