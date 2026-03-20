pub mod brake;
pub mod direction;
pub mod gearset;
pub mod motor_type;

use argparse::{Args, error_msg};
use brake::BrakeModeObj;
use direction::DirectionObj;
use gearset::GearsetObj;
use micropython_rs::{
    class, class_methods,
    except::{attribute_error, type_error},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::{
    math::Direction,
    smart::motor::{Gearset, Motor, SetGearsetError},
};

use crate::{
    devices::{self},
    modvenice::{
        Exception, device_error, motor::motor_type::MotorTypeObj, units::rotation::RotationUnitObj,
    },
    registry::RegistryGuard,
};

#[class(qstr!(Motor))]
#[repr(C)]
pub struct MotorObj {
    base: ObjBase,
    guard: RegistryGuard<'static, Motor>,
}

impl From<SetGearsetError> for Exception {
    fn from(value: SetGearsetError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class_methods]
impl MotorObj {
    #[method(ty = var_between(min = 1, max = 3), binding = "static")]
    fn new_v5(args: &[Obj]) -> Result<Self, Exception> {
        let mut reader = Args::new(args.len(), 0, args).reader();

        let port = reader.next_positional()?;
        let direction = reader.next_positional_or(DirectionObj::FORWARD)?;
        let gearset = reader.next_positional_or(GearsetObj::GREEN)?;

        let guard = devices::lock_port(port, |port| {
            Motor::new(port, gearset.gearset(), direction.direction())
        });

        if guard.borrow().is_exp() {
            // no need to free guard manually
            Err(device_error(c"invalid motor type, expected V5, found Exp"))
        } else {
            Ok(Self {
                base: Self::OBJ_TYPE.into(),
                guard,
            })
        }
    }

    #[method(ty = var_between(min = 1, max = 2), binding = "static")]
    fn new_exp(args: &[Obj]) -> Result<Self, Exception> {
        let mut reader = Args::new(args.len(), 0, args).reader();
        reader.assert_npos(1, 2).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let direction = reader.next_positional_or(DirectionObj::FORWARD)?;

        let guard = devices::lock_port(port, |port| Motor::new_exp(port, direction.direction()));
        if guard.borrow().is_v5() {
            // no need to free guard manually
            Err(device_error(c"invalid motor type, expected Exp, found V5"))
        } else {
            Ok(MotorObj {
                base: Self::OBJ_TYPE.into(),
                guard,
            })
        }
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
    fn set_voltage(&self, volts: f32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_voltage(volts as f64)?)
    }

    #[method]
    fn set_velocity(&self, rpm: i32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_velocity(rpm)?)
    }

    #[method]
    fn brake(&self, mode: &BrakeModeObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().brake(mode.mode())?)
    }

    #[method]
    fn set_gearset(&self, gearset: &GearsetObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_gearset(gearset.gearset())?)
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            attribute_error(c"Motor attributes are read-only").raise(token());
        };
        result.return_value(match attr.as_str() {
            "is_exp" => self.guard.borrow().is_exp().into(),
            "is_v5" => self.guard.borrow().is_v5().into(),
            "max_voltage" => (self.guard.borrow().max_voltage() as f32).into(),
            "motor_type" => {
                Obj::from_static(MotorTypeObj::new_static(self.guard.borrow().motor_type()))
            }
            _ => return,
        });
    }

    #[method]
    fn get_gearset(&self) -> Result<Obj, Exception> {
        let gearset = self.guard.borrow().gearset()?;
        Ok(Obj::from_static(match gearset {
            Gearset::Red => GearsetObj::RED,
            Gearset::Green => GearsetObj::GREEN,
            Gearset::Blue => GearsetObj::BLUE,
        }))
    }

    #[method(ty = var_between(min = 4, max = 4))]
    fn set_position_target(args: &[Obj]) -> Result<(), Exception> {
        let mut reader = Args::new(args.len(), 0, args).reader();

        let motor = reader.next_positional::<&MotorObj>().unwrap();
        let position_val = reader.next_positional()?;
        let unit_obj = reader.next_positional::<&RotationUnitObj>()?;
        let velocity_val = reader.next_positional()?;

        let angle = unit_obj.unit().float_to_angle(position_val);
        motor
            .guard
            .borrow_mut()
            .set_position_target(angle, velocity_val)?;
        Ok(())
    }

    #[method]
    fn get_velocity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().velocity()? as f32)
    }

    #[method]
    fn get_power(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().power()? as f32)
    }

    #[method]
    fn get_torque(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().torque()? as f32)
    }

    #[method]
    fn get_voltage(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().voltage()? as f32)
    }

    #[method]
    fn get_raw_position(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().raw_position()?)
    }

    #[method]
    fn get_current(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().current()? as f32)
    }

    #[method]
    fn get_efficiency(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().efficiency()? as f32)
    }

    #[method]
    fn get_current_limit(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().current_limit()? as f32)
    }

    #[method]
    fn get_voltage_limit(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().voltage_limit()? as f32)
    }

    #[method]
    fn get_temperature(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().temperature()? as f32)
    }

    #[method]
    fn set_profiled_velocity(&self, velocity: i32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_profiled_velocity(velocity)?)
    }

    #[method]
    fn reset_position(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_position()?)
    }

    #[method]
    fn set_current_limit(&self, limit: f32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_current_limit(limit as f64)?)
    }

    #[method]
    fn set_voltage_limit(&self, limit: f32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_voltage_limit(limit as f64)?)
    }

    #[method]
    fn is_over_temperature(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_over_temperature()?)
    }

    #[method]
    fn is_over_current(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_over_current()?)
    }

    #[method]
    fn is_driver_fault(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_driver_fault()?)
    }

    #[method]
    fn is_driver_over_current(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_driver_over_current()?)
    }

    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let angle = self.guard.borrow().position()?;
        Ok(unit.unit().angle_to_float(angle))
    }

    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(position);
        Ok(self.guard.borrow_mut().set_position(angle)?)
    }

    #[method]
    fn set_direction(&self, direction: &DirectionObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_direction(direction.direction())?)
    }

    #[method]
    fn get_direction(&self) -> Result<Obj, Exception> {
        let dir = self.guard.borrow().direction()?;
        Ok(Obj::from_static(match dir {
            Direction::Forward => DirectionObj::FORWARD,
            Direction::Reverse => DirectionObj::REVERSE,
        }))
    }

    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        let status = self.guard.borrow().status()?;
        Ok(status.bits() as i32)
    }

    #[method]
    fn get_faults(&self) -> Result<i32, Exception> {
        let faults = self.guard.borrow().faults()?;
        Ok(faults.bits() as i32)
    }

    #[method]
    fn free(&self) {
        self.guard.free_or_raise();
    }
}
