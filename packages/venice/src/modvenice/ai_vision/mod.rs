pub mod april_tag_family;
pub mod color;
pub mod color_code;
pub mod detection_mode;
pub mod flags;
pub mod object;

use argparse::{Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjType},
    tuple::new_tuple,
};
use vexide_devices::smart::{
    PortError,
    ai_vision::{AiVisionObjectError, AiVisionSensor},
};

use crate::{
    devices::{self},
    modvenice::{
        Exception,
        ai_vision::{
            april_tag_family::AprilTagFamilyObj, color::AiVisionColorObj,
            color_code::AiVisionColorCodeObj, detection_mode::AiVisionDetectionModeObj,
            flags::AiVisionFlagsObj,
        },
        device_error,
    },
    registry::SmartGuard,
};

/// AI Vision Sensor
///
/// This class provides an API for interacting with the AI Vision sensor. The AI Vision sensor is
/// meant to be a direct upgrade from the `VisionSensor` with a wider camera range
/// and AI model capabilities.
///
/// # Hardware overview
///
/// The AI Vision sensor has three detection modes that can all be enabled at the same time:
/// - Color detection (`AiVisionDetectionMode.COLOR`)
/// - Custom model detection (`AiVisionDetectionMode.MODEL`)
/// - AprilTag detection (`AiVisionDetectionMode.APRILTAG`; requires color detection to be enabled)
///
/// Currently there is no known way to upload custom models to the sensor and fields do not have
/// AprilTags. However, there are built-in models that can be used for detection.
///
/// See [VEX's documentation](https://kb.vex.com/hc/en-us/articles/30326315023892-Using-AI-Classifications-with-the-AI-Vision-Sensor)
/// for more information.
///
/// The resolution of the AI Vision sensor is 320x240 pixels. It has a horizontal FOV of 74 degrees
/// and a vertical FOV of 63 degrees. Both of these values are a slight upgrade from the Vision
/// Sensor.
///
/// Unlike the Vision Sensor, the AI Vision sensor uses more human-readable color signatures that
/// may be created without the AI Vision utility, though uploading color signatures with VEX's AI
/// Vision Utility over USB is still an option.
///
/// An AI Vision sensor.
///
/// Object coordinates use pixels with the origin at the image's top-left, positive `x` to the right, and positive `y` downward.
#[class(qstr!(AiVisionSensor))]
#[repr(C)]
pub struct AiVisionSensorObj {
    base: ObjBase,
    guard: SmartGuard<AiVisionSensor>,
}

impl From<AiVisionObjectError> for Exception {
    fn from(value: AiVisionObjectError) -> Self {
        device_error(error_msg!("{value}"))
    }
}

#[class_methods]
impl AiVisionSensorObj {
    /// Creates a new AI Vision sensor from Smart Port `port`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// # Do something with the AI Vision sensor.
    /// ```
    ///
    /// # Raises
    ///
    /// - `ValueError`: If `port` isn't from 1 through 21 or is already in use.
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

        let port = reader.next_positional()?;

        let guard = devices::lock_port(port, AiVisionSensor::new);

        Ok(Self {
            base: ObjBase::new(ty),
            guard,
        })
    }

    /// Returns the current temperature of the sensor in degrees Celsius.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     sensor = AiVisionSensor(1)
    ///     while True:
    ///         print(sensor.get_temperature())
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or its port contains the wrong device type.
    #[method]
    fn get_temperature(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().temperature()? as f32)
    }

    /// Registers a color code association on the sensor.
    ///
    /// Color codes are effectively "groups" of color signatures. A color code associated with multiple
    /// color signatures on the sensor will be detected as a single object when all signatures are seen
    /// next to each other.
    ///
    /// `id` is intended to be in the interval [1, 8], and every signature ID in `code` is intended to be
    /// in [1, 7]. The configured device dependency currently contains a reversed validation check that
    /// rejects valid signature IDs, so this operation isn't usable with a valid color code until that
    /// implementation is fixed. Values outside either documented range aren't safely reported as Python
    /// exceptions and must be rejected by the caller.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// color = AiVisionColor(Color(255, 0, 0), 10.0, 1.0)
    /// sensor.set_color(1, color)
    /// code = AiVisionColorCode(1)
    /// sensor.set_color_code(1, code)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn set_color_code(&self, id: i32, code: &AiVisionColorCodeObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_color_code(id as _, &code.code())?)
    }

    /// Returns the color code set on the AI Vision sensor with the given `id` if it exists.
    ///
    /// Valid color-code slot IDs are 1 through 8. Other values aren't safely reported as Python
    /// exceptions and must be rejected by the caller.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// code = AiVisionColorCode(1)
    /// sensor.set_color_code(1, code)
    ///
    /// code = sensor.get_color_code(1)
    /// if code is not None:
    ///     print(code)
    /// else:
    ///     print("Something went wrong!")
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn get_color_code(&self, id: i32) -> Result<Option<AiVisionColorCodeObj>, Exception> {
        Ok(self
            .guard
            .borrow()
            .color_code(id as _)?
            .map(AiVisionColorCodeObj::new))
    }

    /// Sets a color signature for the AI Vision sensor.
    ///
    /// `id` must be in the interval [1, 7]. Other values aren't safely reported as Python exceptions and
    /// must be rejected by the caller.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// color = AiVisionColor(Color(255, 0, 0), 10.0, 1.0)
    ///
    /// sensor.set_color(1, color)
    /// sensor.set_color(2, color)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn set_color(&self, id: i32, color: &AiVisionColorObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_color(id as _, color.color())?)
    }

    /// Returns the color signature set on the AI Vision sensor with the given `id` if it exists.
    ///
    /// `id` must be in the interval [1, 7]. Other values aren't safely reported as Python exceptions and
    /// must be rejected by the caller.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// color = AiVisionColor(Color(255, 0, 0), 10.0, 1.0)
    /// sensor.set_color(1, color)
    ///
    /// color = sensor.get_color(1)
    /// if color is not None:
    ///     print(color)
    /// else:
    ///     print("Something went wrong!")
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn get_color(&self, id: i32) -> Result<Option<AiVisionColorObj>, Exception> {
        Ok(self
            .guard
            .borrow()
            .color(id as _)?
            .map(AiVisionColorObj::new))
    }

    /// Sets the detection mode of the AI Vision sensor.
    ///
    /// Combine `AiVisionDetectionMode` constants with `|`. AprilTag detection requires color detection to
    /// be enabled. The current overlay settings are preserved.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// sensor.set_detection_mode(AiVisionDetectionMode.COLOR | AiVisionDetectionMode.COLOR_MERGE)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn set_detection_mode(&self, mode: &AiVisionDetectionModeObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_detection_mode(mode.mode())?)
    }

    /// Returns the current flags of the AI Vision sensor including the detection mode flags set by
    /// `AiVisionSensor.set_detection_mode`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// print(sensor.get_flags())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn get_flags(&self) -> Result<AiVisionFlagsObj, Exception> {
        Ok(AiVisionFlagsObj::new(self.guard.borrow().flags()?))
    }

    /// Sets the full flags of the AI Vision sensor, including the detection mode.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// # Enable all detection modes except for custom model and disable USB overlay.
    /// flags = AiVisionFlags.DISABLE_USB_OVERLAY | AiVisionFlags.DISABLE_MODEL
    /// sensor.set_flags(flags)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn set_flags(&self, flags: &AiVisionFlagsObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_flags(flags.flags())?)
    }

    /// Restarts the automatic white balance process.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn start_awb(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().start_awb()?)
    }

    /// Unknown use.
    ///
    /// `test` is passed to the undocumented VEX test mode using its low eight bits. This firmware-facing
    /// operation normally shouldn't be used by application code.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn enable_test(&self, test: i32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().enable_test(test as u8)?)
    }

    /// Sets the AprilTag family that the sensor will try to detect.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// sensor.set_apriltag_family(AprilTagFamily.TAG16H5)
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn set_apriltag_family(&self, family: &AprilTagFamilyObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_apriltag_family(family.family())?)
    }

    /// Returns the number of objects currently detected by the AI Vision sensor.
    ///
    /// The sensor can report at most 24 objects at once.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     sensor = AiVisionSensor(1)
    ///     while True:
    ///         print("AI Vision sensor currently detects {} objects".format(sensor.get_object_count()))
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    fn get_object_count(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().object_count()? as i32)
    }

    /// Returns all objects detected by the AI Vision sensor.
    ///
    /// Each tuple item is an `AiVisionColorObject`, `AiVisionCodeObject`, `AiVisionAprilTagObject`, or
    /// `AiVisionModelObject`, according to the active detection modes.
    ///
    /// # Examples
    ///
    /// Loop through all objects of a specific type:
    ///
    /// ```python
    /// from venice import *
    ///
    /// async def main():
    ///     sensor = AiVisionSensor(1)
    ///     while True:
    ///         objects = sensor.get_objects()
    ///         for obj in objects:
    ///             if isinstance(obj, AiVisionColorObject):
    ///                 print(Point2(obj.x, obj.y))
    ///         await vasyncio.Sleep(10, MILLIS)
    ///
    /// vasyncio.run(main())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected, the port contains the wrong device type, or the
    ///   sensor reports an invalid object or model classification.
    #[method]
    #[stub(
        sig = "(self) -> tuple[AiVisionColorObject | AiVisionCodeObject | AiVisionAprilTagObject | AiVisionModelObject, ...]"
    )]
    fn get_objects(&self) -> Result<Obj, Exception> {
        let objects = self.guard.borrow().objects()?;
        let objects = objects
            .into_iter()
            .map(object::create_obj)
            .collect::<Vec<_>>();
        Ok(new_tuple(&objects[..]))
    }

    /// Returns all color codes set on the AI Vision sensor.
    ///
    /// The current implementation starts at invalid slot 0 and queries only seven slots, while the
    /// hardware slots are numbered 1 through 8. This method therefore can't currently complete as
    /// documented.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = AiVisionSensor(1)
    /// sensor.set_color_code(1, AiVisionColorCode(1))
    /// sensor.set_color_code(2, AiVisionColorCode(1, 2))
    ///
    /// print(sensor.get_color_codes())
    /// ```
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If the sensor is disconnected or the port contains the wrong device type.
    #[method]
    #[stub(sig = "(self) -> tuple[AiVisionColorCode | None, ...]")]
    fn get_color_codes(&self) -> Result<Obj, Exception> {
        let guard = self.guard.borrow();
        let codes = (0..7)
            .map(|n| guard.color_code(n))
            .map(|code| code.map(|code| Obj::from(code.map(AiVisionColorCodeObj::new))))
            .collect::<Result<Vec<_>, PortError>>()?;
        Ok(new_tuple(&codes[..]))
    }

    /// Releases this sensor and frees its Smart Port lock.
    ///
    /// The object is unusable afterward, but its Smart Port can be assigned to another device.
    ///
    /// # Raises
    ///
    /// - `ValueError`: If the sensor has already been freed.
    #[method]
    #[stub(sig = "(self) -> None")]
    fn free(&self) -> Obj {
        self.guard.free_or_raise();
        Obj::NONE
    }
}
