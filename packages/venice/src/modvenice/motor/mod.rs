pub mod brake;
pub mod direction;
pub mod gearset;
pub mod motor_type;

use argparse::{Args, error_msg};
use brake::BrakeModeObj;
use direction::DirectionObj;
use gearset::GearsetObj;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::type_error,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use vexide_devices::{
    math::Direction,
    smart::{
        SmartDevice,
        motor::{Gearset, Motor, SetGearsetError},
    },
};

use crate::{
    devices::{self},
    modvenice::{
        Exception, device_error, motor::motor_type::MotorTypeObj, read_only_attr::read_only_attr,
        units::rotation::RotationUnitObj,
    },
    registry::SmartGuard,
};

/// A motor plugged into a Smart Port.
///
/// This class provides abstractions for interacting with VEX Smart Motors, supporting both the 11W
/// and 5.5W variants.
///
/// # Overview
///
/// The V5 Smart Motors come in two variants: [an 11W model](https://www.vexrobotics.com/276-4840.html),
/// with interchangeable gear cartridges and [a 5.5W model](https://www.vexrobotics.com/276-4842.html),
/// with a fixed gearing. The 11W motor supports three cartridge options, which will gear the motor
/// down from its base RPM of 3600: a red cartridge providing 100 RPM output, a green cartridge for
/// 200 RPM, and a blue cartridge for 600 RPM. The 5.5W motor comes with a non-interchangeable 200
/// RPM gear cartridge.
///
/// Smart Motors feature several integrated sensors, including an encoder for measuring the velocity
/// and position of the motor, a temperature sensor for detecting overheats, and sensors for
/// measuring output voltage, current, and efficiency.
///
/// Communication between a Smart motor and the V5 Brain occur at two different intervals. While
/// the motor communicates with the Brain every 5 milliseconds (and commands can be written to the
/// motor every 5mS), the Brain only reads data from the motor every 10mS. This effectively places
/// the date *write* interval at 5mS and the data *read* interval at 10mS.
///
/// More in-depth specs for the 11W motor can be found [here](https://kb.vex.com/hc/en-us/articles/360060929971-Understanding-V5-Smart-Motors).
///
/// # Current Limitations
///
/// There are some cases where VEXos or the motor itself may decide to limit output current:
///
/// - **Stall Prevention**: The stall current on 11W motors is limited to 2.5A. This limitation
///   eliminates the need for automatic resetting fuses (PTC devices) in the motor, which can
///   disrupt operation. By restricting the stall current to 2.5A, the motor effectively avoids
///   undesirable performance dips and ensures that users do not inadvertently cause stall
///   situations.
///
/// - **Motor Count**: Robots that use 8 or fewer 11W motors will have the aforementioned current limit
///   of 2.5A set for each motor. Robots using more than 8 motors, will have a lower default current limit
///   per-motor than 2.5A. This limit is determined in VEXos by a calculation accounting for the number of
///   motors plugged in, and the user's manually set current limits using `Motor.set_current_limit`. For
///   more information regarding the current limiting behavior of VEXos, see [this forum post](https://www.vexforum.com/t/how-does-the-decreased-current-affect-the-robot-when-using-more-than-8-motors/72650/4).
///
/// - **Temperature Management**: Motors have an onboard sensor for measuring internal temperature.
///   If the motor determines that it is overheating, it will throttle its output current and warn
///   the user.
///
/// # Motor Control
///
/// Each motor contains a sophisticated control system built around a Cortex M0+ microcontroller.
/// The microcontroller continuously monitors position, speed, direction, voltage, current, and
/// temperature through integrated sensors.
///
/// The onboard motor firmware implements a set of pre-tuned PID (Proportional-Integral-Derivative)
/// controllers operating on a 10-millisecond cycle for position and velocity control. Motors also
/// have braking functionality for holding a specific position under load.
#[class(qstr!(Motor))]
#[repr(C)]
pub struct MotorObj {
    base: ObjBase,
    guard: SmartGuard<Motor>,
}

impl From<SetGearsetError> for Exception {
    fn from(value: SetGearsetError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class_methods]
impl MotorObj {
    /// The maximum voltage value that can be sent to a V5 `Motor`.
    #[constant]
    const V5_MAX_VOLTAGE: f32 = Motor::V5_MAX_VOLTAGE as f32;
    /// The maximum voltage value that can be sent to an EXP `Motor`.
    #[constant]
    const EXP_MAX_VOLTAGE: f32 = Motor::EXP_MAX_VOLTAGE as f32;
    /// The interval at which the Brain will send new packets to a `Motor`, in milliseconds.
    #[constant]
    const WRITE_INTERVAL_MS: i32 = Motor::UPDATE_INTERVAL.as_millis() as i32;

    /// Creates a new 11W (V5) Smart Motor.
    ///
    /// See `Motor.new_exp` to create a 5.5W (EXP) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor.new_v5(1)
    /// assert motor.is_v5
    /// assert motor.max_voltage == Motor.V5_MAX_VOLTAGE
    /// ```
    // Do NOT rearrange the order of these attributes!!! The build script depends on it
    #[stub(
        sig = "(port: int, direction: Direction = Direction.FORWARD, gearset: Gearset = Gearset.GREEN) -> Motor"
    )]
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

    /// Creates a new 5.5W (EXP) Smart Motor.
    ///
    /// See `Motor()` or `Motor.new_v5` to create a 11W (V5) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// assert motor.is_exp
    /// assert motor.max_voltage == Motor.EXP_MAX_VOLTAGE
    /// ```
    #[method(ty = var_between(min = 1, max = 2), binding = "static")]
    #[stub(sig = "(port: int, direction: Direction = Direction.FORWARD) -> Motor")]
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

    /// Creates a new 11W (V5) Smart Motor.
    ///
    /// See `Motor.new_exp` to create a 5.5W (EXP) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// assert motor.is_v5
    /// assert motor.max_voltage == Motor.V5_MAX_VOLTAGE
    /// ```
    #[make_new]
    #[stub(
        sig = "(self, port: int, direction: Direction = Direction.FORWARD, gearset: Gearset = Gearset.GREEN) -> None"
    )]
    fn make_new(_: &ObjType, _: usize, n_kw: usize, args: &[Obj]) -> Result<Self, Exception> {
        if n_kw != 0 {
            Err(type_error(c"function does not accept keyword arguments").into())
        } else {
            Self::new_v5(args)
        }
    }

    /// Sets the motor's output voltage.
    ///
    /// This voltage value spans from -12 (fully spinning reverse) to +12 (fully spinning forwards)
    /// volts, and controls the raw output of the motor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Give the motor full power:
    ///
    /// ```python
    /// from venice import *
    ///
    /// v5_motor = Motor(1)
    /// exp_motor = Motor(2)
    ///
    /// v5_motor.set_voltage(v5_motor.max_voltage)
    /// exp_motor.set_voltage(exp_motor.max_voltage)
    /// ```
    ///
    /// Drive the motor based on a controller joystick:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// controller = Controller()
    ///
    /// async def main():
    ///     while True:
    ///         controller_state = controller.get_state()
    ///         voltage = controller_state.left_stick.x * motor.max_voltage
    ///
    ///         motor.set_voltage(voltage)
    ///
    ///         await vasyncio.Sleep(Motor.WRITE_INTERVAL_MS, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn set_voltage(&self, volts: f32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_voltage(volts as f64)?)
    }

    /// Spins the motor at a target velocity.
    ///
    /// This velocity corresponds to different actual speeds in RPM depending on the gearset used
    /// for the motor. Velocity is held with an internal PID controller to ensure consistent
    /// speed, as opposed to setting the motor's voltage.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Spin a motor at 100 RPM:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.set_velocity(100)
    /// ```
    #[method]
    fn set_velocity(&self, rpm: i32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_velocity(rpm)?)
    }

    /// Stops this motor with the given `BrakeMode`.
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
    /// motor = Motor(1)
    /// motor.brake(BrakeMode.HOLD)
    /// ```
    #[method]
    fn brake(&self, mode: &BrakeModeObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().brake(mode.mode())?)
    }

    /// Sets the gearset of an 11W motor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If the motor is a 5.5W EXP Smart Motor, which has no swappable gearset, or if
    /// no motor is connected to the port.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # This must be a V5 motor
    /// motor = Motor(1, Direction.FORWARD, Gearset.GREEN)
    ///
    /// # Set the motor to use the red gearset
    /// motor.set_gearset(Gearset.RED)
    /// ```
    #[method]
    fn set_gearset(&self, gearset: &GearsetObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_gearset(gearset.gearset())?)
    }

    #[attr]
    #[stub(attrs = ["is_exp: bool", "is_v5: bool", "max_voltage: float", "motor_type: MotorType"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
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

    /// Returns the gearset of the motor
    ///
    /// For 5.5W motors, this will always be returned as `Gearset::Green`.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the gearset of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// def print_gearset(motor: Motor):
    ///     try:
    ///         gearset = motor.gearset()
    ///     except:
    ///         print("Failed to get gearset. Is this an EXP motor?")
    ///         return
    ///
    ///     if gearset == Gearset.RED:
    ///         print("Motor is using the red gearset")
    ///     elif gearset == Gearset.GREEN:
    ///         print("Motor is using the green gearset")
    ///     elif gearset == Gearset.BLUE:
    ///         print("Motor is using the blue gearset")
    /// ```
    #[method]
    #[stub(sig = "(self) -> Gearset")]
    fn get_gearset(&self) -> Result<Obj, Exception> {
        let gearset = self.guard.borrow().gearset()?;
        Ok(Obj::from_static(match gearset {
            Gearset::Red => GearsetObj::RED,
            Gearset::Green => GearsetObj::GREEN,
            Gearset::Blue => GearsetObj::BLUE,
        }))
    }

    /// Sets an absolute position target for the motor to attempt to reach.
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
    /// motor = Motor(1)
    ///
    /// # Turn the motor to a position of 90 degrees at 200rpm.
    /// motor.set_position_target(90, DEGREES, 200)
    /// ```
    #[method(ty = var_between(min = 4, max = 4))]
    #[stub(sig = "(self, position: float, unit: RotationUnit, velocity: int) -> None")]
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

    /// Returns the motor's estimate of its angular velocity in rotations per minute (RPM).
    ///
    /// # Accuracy
    ///
    /// In some cases, this reported value may be noisy or inaccurate, especially for systems where
    /// accurate velocity control at high speeds is required (such as flywheels).
    // TODO: add Motor::timestamp then add this clause to the previous paragraph
    // If the
    // accuracy of this value proves inadequate, you may opt to perform your own velocity
    // calculations by differentiating [`Motor::position`] over the reported internal timestamp
    // of the motor using [`Motor::timestamp`].
    //
    // > For more information about Smart motor velocity estimation, see [this article](https://sylvie.fyi/sylib/docs/db/d8e/md_module_writeups__velocity__estimation.html).
    //
    // also omitted
    //
    // # Note
    //
    // To get the current **target** velocity instead of the estimated velocity, use
    // [`Motor::target`].
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Get the current velocity of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// print(motor.get_velocity())
    /// ```
    //
    // TODO: Add time API and add this example
    // Calculate acceleration of a motor:
    //
    // ```no_run
    // use std::time::Instant;
    //
    // use vexide::prelude::*;
    //
    // #[vexide::main]
    // async fn main(peripherals: Peripherals) {
    //     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    //
    //     let mut last_velocity = motor.velocity().unwrap();
    //     let mut start_time = Instant::now();
    //
    //     loop {
    //         let velocity = motor.velocity().unwrap();
    //
    //         // Make sure we don't divide by zero
    //         let elapsed = start_time.elapsed().as_secs_f64() + 0.001;
    //
    //         // Calculate acceleration
    //         let acceleration = (velocity - last_velocity) / elapsed;
    //         println!(
    //             "Velocity: {:.2} RPM, Acceleration: {:.2} RPM/s",
    //             velocity, acceleration
    //         );
    //
    //         last_velocity = velocity;
    //         start_time = Instant::now();
    //
    //         sleep(Motor::UPDATE_INTERVAL).await;
    //     }
    // }
    // ```
    #[method]
    fn get_velocity(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().velocity()? as f32)
    }

    /// Returns the power drawn by the motor in Watts.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the power drawn by a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         power = motor.get_power()
    ///         print(f"Power: {power:.2}")
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    // Overrated csm character dont ever call ts
    #[method]
    fn get_power(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().power()? as f32)
    }

    /// Returns the torque output of the motor in Nm.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the torque output of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         torque = motor.get_torque()
    ///         print(f"Torque: {torque:.2}Nm")
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_torque(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().torque()? as f32)
    }

    /// Returns the voltage the motor is drawing in volts.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the voltage drawn by a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         voltage = motor.get_voltage()
    ///         print(f"Voltage: {voltage:.2}V")
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_voltage(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().voltage()? as f32)
    }

    /// Returns the most recently recorded raw encoder tick data from the motor's IME.
    ///
    /// The motor's integrated encoder has a TPR of 4096. Gearset is not taken into consideration
    /// when dealing with the raw value, meaning this measurement will be taken relative to the
    /// motor's internal position *before* being geared down from 3600RPM.
    ///
    /// Methods such as `Motor::reset_position` and `Motor::set_position` do not
    /// change the value of this raw measurement.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```python
    /// use vexide::prelude::*;
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         raw_pos = motor.get_raw_position()
    ///         print(f"Raw Position: {raw_pos}")
    ///
    ///         await vasyncio.sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_raw_position(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().raw_position()?)
    }

    /// Returns the electrical current draw of the motor in amps.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the current draw of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(motor.max_voltage)
    ///     while True:
    ///         current = motor.get_current()
    ///         print(f"Current: {current:.2}A")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_current(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().current()? as f32)
    }

    /// Returns the efficiency of the motor from a range of [0.0, 1.0].
    ///
    /// An efficiency of 1.0 means that the motor is moving electrically while drawing no electrical
    /// power, and an efficiency of 0.0 means that the motor is drawing power but not moving.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the efficiency of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(motor.max_voltage)
    ///     while True:
    ///         efficiency = motor.get_efficiency()
    ///         print(f"Current: {efficiency:.2}")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    /// ```
    #[method]
    fn get_efficiency(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().efficiency()? as f32)
    }

    /// Returns the current limit for the motor in amps.
    ///
    /// This limit can be configured with the `Motor.set_current_limit` method.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the current limit of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// current_limit = motor.get_current_limit()
    /// print(f"Current Limit: {current_limit:.2}A")
    /// ```
    #[method]
    fn get_current_limit(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().current_limit()? as f32)
    }

    /// Returns the voltage limit for the motor if one has been explicitly set.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the voltage limit of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// voltage_limit = motor.get_voltage_limit()
    /// print(f"Voltage Limit: {voltage_limit:.2}V")
    /// ```
    #[method]
    fn get_voltage_limit(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().voltage_limit()? as f32)
    }

    /// Returns the internal temperature recorded by the motor in increments of 5 °C.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Turn off the motor if it gets too hot:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(12)
    ///     while True:
    ///         if motor.get_temperature() > 30:
    ///             motor.brake(BrakeMode.COAST)
    ///         else:
    ///             motor.set_voltage(12)
    ///         await vasyncio.Sleep(10, MILLIS)
    /// ```
    #[method]
    fn get_temperature(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().temperature()? as f32)
    }

    /// Changes the output velocity for a profiled movement (motor_move_absolute or
    /// motor_move_relative).
    ///
    /// This will have no effect if the motor is not following a profiled movement.
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
    /// motor = Motor(1)
    ///
    /// # Set the motor's target to a Position so that changing the velocity isn't a noop.
    /// motor.set_position_target(90, DEGREES, 200)
    /// motor.set_profiled_velocity(100)
    /// ```
    #[method]
    fn set_profiled_velocity(&self, velocity: i32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_profiled_velocity(velocity)?)
    }

    /// Sets the current encoder position to zero without moving the motor.
    ///
    /// Analogous to taring or resetting the encoder to the current position.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Move the motor in increments of 10 degrees:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         motor.set_position_target(10, DEGREES, 200)
    ///         await vasyncio.Sleep(1, SECOND)
    ///         motor.reset_position()
    /// ```
    #[method]
    fn reset_position(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().reset_position()?)
    }

    /// Sets the current limit for the motor in amps.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Limit the current draw of a motor to 2.5A:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.set_current_limit(2.5)
    /// ```
    #[method]
    fn set_current_limit(&self, limit: f32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_current_limit(limit as f64)?)
    }

    /// Sets the voltage limit for the motor in volts.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Limit the voltage of a motor to 4V:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.set_voltage_limit(4)
    /// # Will appear as if the voltage was set to only 4V
    /// motor.set_voltage(12)
    /// ```
    #[method]
    fn set_voltage_limit(&self, limit: f32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_voltage_limit(limit as f64)?)
    }

    /// Returns `True` if the motor's over temperature flag is set.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Turn off the motor if it gets too hot:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(12)
    ///     while True:
    ///         if motor.is_over_temperature():
    ///             motor.brake(BrakeMode.COAST)
    ///         else:
    ///             motor.set_voltage(12)
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn is_over_temperature(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_over_temperature()?)
    }

    /// Returns `True` if the motor's over-current flag is set.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print a warning if the motor draws too much current:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(12)
    ///     while True:
    ///         if motor.is_over_current():
    ///             print("Warning: Motor is drawing too much current")
    ///         current = motor.get_current()
    ///         print(f"Current: {current:.2}A")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn is_over_current(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_over_current()?)
    }

    /// Returns `True` if a H-bridge (motor driver) fault has occurred.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print a warning if the motor's H-bridge has a fault:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(12)
    ///     while True:
    ///         if motor.is_driver_fault():
    ///             print("Warning: Motor has a H-bridge fault")
    ///         current = motor.get_current()
    ///         print(f"Current: {current:.2}A")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn is_driver_fault(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_driver_fault()?)
    }

    /// Returns `True` if the motor's H-bridge has an over-current fault.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print a warning if it draws too much current:
    ///
    /// ```no_run
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     motor.set_voltage(12)
    ///     while True:
    ///         if motor.is_driver_over_current():
    ///             print("Warning: Motor is drawing too much current")
    ///         current = motor.get_current()
    ///         print(f"Current: {current:.2}A")
    ///
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn is_driver_over_current(&self) -> Result<bool, Exception> {
        Ok(self.guard.borrow().is_driver_over_current()?)
    }

    /// Returns the angular position of the motor as measured by the IME (integrated motor encoder).
    ///
    /// # Gearing affects position!
    ///
    /// Position measurements are dependent on the Motor's `Gearset`, and may be reported
    /// incorrectly if the motor is configured with the incorrect gearset variant. Make sure
    /// that the motor is configured with the same gearset as its physical cartridge color.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Print the current position of a motor:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         position = motor.get_position(DEGREES)
    ///         print(f"Position: {position}")
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_position(&self, unit: &RotationUnitObj) -> Result<f32, Exception> {
        let angle = self.guard.borrow().position()?;
        Ok(unit.unit().angle_to_float(angle))
    }

    /// Sets the current encoder position to the given position without moving the motor.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given
    /// position.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Set the current position of the motor to 90 degrees:
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.set_position(90, DEGREES)
    /// ```
    ///
    /// Reset the position of the motor to 0 degrees (analogous to `Motor.reset_position`):
    ///
    /// ```python
    /// from venice import *
    ///
    /// motor = Motor(1)
    /// motor.set_position(0, DEGREES)
    /// ```
    #[method]
    fn set_position(&self, position: f32, unit: &RotationUnitObj) -> Result<(), Exception> {
        let angle = unit.unit().float_to_angle(position);
        Ok(self.guard.borrow_mut().set_position(angle)?)
    }

    /// Sets the motor to operate in a given `Direction`.
    ///
    /// This determines which way the motor considers to be “forwards”. You can use the marking on
    /// the back of the motor as a reference:
    ///
    /// - When `Direction.FORWARD` is specified, positive velocity/voltage values will cause the
    ///   motor to rotate **with the arrow on the back**. Position will increase as the motor
    ///   rotates **with the arrow**.
    /// - When `Direction::REVERSE` is specified, positive velocity/voltage values will cause the
    ///   motor to rotate **against the arrow on the back**. Position will increase as the motor
    ///   rotates **against the arrow**.
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
    /// motor = Motor(1, Direction.FORWARD)
    /// motor.set_direction(Direction.REVERSE)
    /// ```
    #[method]
    fn set_direction(&self, direction: &DirectionObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_direction(direction.direction())?)
    }

    /// Returns the `Direction` of this motor.
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
    /// def print_motor_direction(motor: Motor):
    ///     if motor.get_direction() == Direction.FORWARD:
    ///         print("Motor is set to forwards")
    ///     else:
    ///         print("Motor is set to reverse")
    /// ```
    #[method]
    #[stub(sig = "(self) -> Direction")]
    fn get_direction(&self) -> Result<Obj, Exception> {
        let dir = self.guard.borrow().direction()?;
        Ok(Obj::from_static(match dir {
            Direction::Forward => DirectionObj::FORWARD,
            Direction::Reverse => DirectionObj::REVERSE,
        }))
    }

    /// Returns the status flags of a motor.
    ///
    /// # Errors
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Check if a motor is "busy" (busy only occurs if communicating with the motor fails)
    ///
    /// ```python
    /// from venice import *
    ///
    /// BUSY_FLAG = 0x01
    ///
    /// def is_motor_busy(motor: Motor) -> bool:
    ///     # Test if the motor status bits contain `BUSY_FLAG`
    ///     motor.get_status() & BUSY_FLAG == BUSY_FLAG
    /// ```
    #[method]
    fn get_status(&self) -> Result<i32, Exception> {
        let status = self.guard.borrow().status()?;
        Ok(status.bits() as i32)
    }

    /// Returns the fault flags of the motor.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    ///
    /// # Examples
    ///
    /// Check if a motor is over temperature:
    ///
    /// ```python
    /// from venice import *
    ///
    /// OVER_TEMPERATURE_FLAG = 0x01
    ///
    /// motor = Motor(1)
    ///
    /// async def main():
    ///     while True:
    ///         faults = motor.get_faults()
    ///         print("Faults: {faults:b}")
    ///
    ///         # Test if `faults` contains `OVER_TEMPERATURE_FLAG`
    ///         if faults & OVER_TEMPERATURE_FLAG == OVER_TEMPERATURE_FLAG:
    ///             print("Warning: Motor is over temperature!")
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_faults(&self) -> Result<i32, Exception> {
        let faults = self.guard.borrow().faults()?;
        Ok(faults.bits() as i32)
    }

    /// Release this motor and free its Smart Port lock. This binding will become unusable after
    /// this call, but you can reuse the underlying Smart Port to construct a new device.
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
