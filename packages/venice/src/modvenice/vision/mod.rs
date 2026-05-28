pub mod code;
pub mod led_mode;
pub mod mode;
pub mod object;
pub mod signature;
pub mod source;
pub mod white_balance;

use argparse::{ArgParser, Args, DefaultParser, IntParser, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjType},
    tuple::new_tuple,
};
use vexide_devices::smart::vision::{
    VisionMode, VisionObjectError, VisionSensor, VisionSignatureError,
};

use crate::{
    devices::{self},
    modvenice::{
        DEVICE_ERROR_TYPE, Exception,
        vision::{
            code::VisionCodeObj, led_mode::LedModeArg, mode::VisionModeObj,
            object::VisionObjectObj, signature::VisionSignatureObj, white_balance::WhiteBalanceArg,
        },
    },
    registry::SmartGuard,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignatureId(u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SignatureIdParser;

impl SignatureId {
    pub fn id(self) -> u8 {
        self.0
    }
}

impl<'a> ArgParser<'a> for SignatureIdParser {
    type Output = SignatureId;

    fn parse(&self, obj: &'a Obj) -> Result<Self::Output, argparse::ParseError> {
        IntParser::new(1..=7).parse(obj).map(SignatureId)
    }
}

impl DefaultParser<'_> for SignatureId {
    type Parser = SignatureIdParser;
}

impl From<VisionObjectError> for Exception {
    fn from(value: VisionObjectError) -> Self {
        Self::new(&DEVICE_ERROR_TYPE, error_msg!("{value}"))
    }
}

impl From<VisionSignatureError> for Exception {
    fn from(value: VisionSignatureError) -> Self {
        Self::new(&DEVICE_ERROR_TYPE, error_msg!("{value}"))
    }
}

/// A Vision Sensor plugged into a Smart Port.
///
/// This class provides an interface for interacting with the VEX Vision Sensor.
///
/// # Hardware Overview
///
/// The VEX Vision Sensor is a device powered by an ARM Cortex M4 and Cortex M0 coprocessor with a
/// color camera for the purpose of performing object recognition. The sensor can be trained to
/// locate objects by color. The camera module itself is very similar internally to the Pixy2
/// camera, and performs its own onboard image processing. Manually processing raw image data from
/// the sensor is not currently possible.
///
/// Every 20 milliseconds, the camera provides a list of the objects found matching up to seven
/// unique `VisionSignature`s. The object’s height, width, and location is provided. Multi-colored
/// objects may also be programmed through the use of `VisionCode`s.
///
/// The Vision Sensor has USB for a direct connection to a computer, where it can be configured
/// using VEX's proprietary vision utility tool to generate color signatures. The Vision Sensor also
/// has Wi-Fi Direct and can act as web server, allowing a live video feed of the camera from any
/// computer equipped with a browser and Wi-Fi.
#[class(qstr!(VisionSensor))]
pub struct VisionSensorObj {
    base: ObjBase,
    guard: SmartGuard<VisionSensor>,
}

#[class_methods]
impl VisionSensorObj {
    /// The horizontal resolution of the vision sensor.
    ///
    /// This value is based on the `VISION_FOV_WIDTH` macro constant in PROS.
    #[constant]
    const HORIZONTAL_RESOLUTION: i32 = VisionSensor::HORIZONTAL_RESOLUTION as i32;

    /// The vertical resolution of the vision sensor.
    ///
    /// This value is based on the `VISION_FOV_HEIGHT` macro constant in PROS.
    #[constant]
    const VERTICAL_RESOLUTION: i32 = VisionSensor::VERTICAL_RESOLUTION as i32;

    /// The horizontal FOV of the vision sensor in degrees.
    #[constant]
    const HORIZONTAL_FOV: f32 = VisionSensor::HORIZONTAL_FOV;

    /// The vertical FOV of the vision sensor in degrees.
    #[constant]
    const VERTICAL_FOV: f32 = VisionSensor::VERTICAL_FOV;

    /// The diagonal FOV of the vision sensor in degrees.
    #[constant]
    const DIAGONAL_FOV: f32 = VisionSensor::DIAGONAL_FOV;

    /// The update rate of the vision sensor, in milliseconds.
    #[constant]
    const UPDATE_INTERVAL_MS: i32 = VisionSensor::UPDATE_INTERVAL.as_millis() as i32;

    /// Creates a new vision sensor.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
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

        let port = reader.next_positional()?;
        let guard = devices::lock_port(port, |p| VisionSensor::new(p));

        Ok(Self {
            base: ObjBase::new(ty),
            guard,
        })
    }

    /// Adds a detection signature to the sensor's onboard memory.
    ///
    /// This signature will be used to identify objects when using `VisionSensor.get_objects`.
    ///
    /// The sensor can store up to 7 unique signatures, with each signature slot denoted by the `id`
    /// parameter. If a signature with an ID matching an existing signature on the sensor is added,
    /// then the existing signature will be overwritten with the new one.
    ///
    /// # Volatile Memory
    ///
    /// The memory on the Vision Sensor is *volatile* and will therefore be wiped when the sensor
    /// loses power. As a result, this function should be called every time the sensor is used on
    /// program start.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    /// `ValueError`: If the given signature ID is not in the interval [1, 7].
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// # These signatures can be generated using VEX's vision utility
    /// example_signature = VisionSignature(10049, 11513, 10781, -425, 1, -212, 4.1)
    ///
    /// # Set signature 1 on the sensor.
    /// sensor.set_signature(1, example_signature)
    /// ```
    #[method]
    #[stub(sig = "(self, id: int, signature: VisionSignature) -> None")]
    fn set_signature(
        &self,
        id: SignatureId,
        signature: &VisionSignatureObj,
    ) -> Result<(), Exception> {
        self.guard
            .borrow_mut()
            .set_signature(id.id(), signature.signature())?;
        Ok(())
    }

    /// Returns a signature from the sensor's onboard volatile memory.
    ///
    /// # Raises
    ///
    /// `DeviceError`: If no device is connected to the port, or if the wrong type of device is
    /// connected.
    /// `ValueError`: If the given signature ID is not in the interval [1, 7].
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// # Set an example signature in the sensor's first slot.
    /// sensor.set_signature(1, VisionSignature(10049, 11513, -425, 1, -212, 4.1))
    ///
    /// # Read signature 1 off the sensor.
    /// # This should be the same as the one we just set.
    /// signature = sensor.get_signature(1)
    ///
    /// print(f"u_min: {signature.u_min}")
    /// print(f"u_max: {signature.u_max}")
    /// print(f"u_mean: {signature.u_mean}")
    /// # etc...
    /// ```
    #[method]
    #[stub(sig = "(self, id: int) -> VisionSignature | None")]
    fn get_signature(&self, id: SignatureId) -> Result<Option<VisionSignatureObj>, Exception> {
        Ok(self
            .guard
            .borrow()
            .signature(id.id())?
            .map(VisionSignatureObj::new))
    }

    /// Returns all signatures currently stored on the sensor's onboard volatile memory.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there was not a sensor connected to the port, or if a read operation
    /// failed, or there was no signature previously set in the slot(s) specified in the
    /// `VisionCode`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// # A bunch of random color signatures.
    /// sig_1 = VisionSignature(10049, 11513, 10781, -425, 1, -212, 4.1)
    /// sig_2 = VisionSignature(8973, 11143, 10058, -2119, -1053, -1586, 5.4)
    /// sig_3 = VisionSignature(-3665, -2917, -3292, 4135, 10193, 7164, 2.0)
    /// sig_4 = VisionSignature(-5845, -4809, -5328, -5495, -4151, -4822, 3.1)
    ///
    /// # Set signatures 1-4.
    /// sensor.set_signature(1, sig_1)
    /// sensor.set_signature(2, sig_2)
    /// sensor.set_signature(3, sig_3)
    /// sensor.set_signature(4, sig_4)
    ///
    /// # Read back the signatures from the sensor's memory.
    /// # These should be the signatures that we just set.
    /// signatures = sensor.get_signatures()
    /// for signature in signatures:
    ///     if signature:
    ///         print("Found sig saved on sensor:")
    ///         print(f"u_min: {signature.u_min}")
    ///         print(f"u_max: {signature.u_max}")
    ///         print(f"u_mean: {signature.u_mean}")
    ///         # etc...
    /// ```
    #[method]
    #[stub(sig = "(self) -> tuple[VisionSignature | None, ...]")]
    fn get_signatures(&self) -> Result<Obj, Exception> {
        let vec = self
            .guard
            .borrow()
            .signatures()?
            .into_iter()
            .map(|s| s.map(VisionSignatureObj::new))
            .map(Obj::from)
            .collect::<Vec<_>>();
        Ok(new_tuple(&vec))
    }

    /// Registers a color code to the sensor's onboard memory. This code will be used to identify
    /// objects when using `VisionSensor.get_objects`.
    ///
    /// Color codes are effectively "signature groups" that the sensor will use to identify objects
    /// containing the color of their signatures next to each other.
    ///
    /// # Volatile Memory
    ///
    /// The onboard memory of the Vision Sensor is *volatile* and will therefore be wiped when the
    /// sensor loses its power source. As a result, this function should be called every time the
    /// sensor is used on program start.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there was not a sensor connected to the port, or if a read operation
    /// failed, or there was no signature previously set in the slot(s) specified in the
    /// `VisionCode`.
    /// `ValueError`: If the given signature ID is not in the interval [1, 7].
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// # Two color signatures.
    /// sig_1 = VisionSensor(10049, 11513, 10781, -425, 1, -212, 4.1)
    /// sig_2 = VisionSensor(8973, 11143, 10058, -2119, -1053, -1586, 5.4)
    ///
    /// # Store the signatures on the sensor.
    /// sensor.set_signature(1, sig_1)
    /// sensor.set_signature(2, sig_2)
    ///
    /// # Create a code associating signatures 1 and 2 together.
    /// code = VisionCode(1, 2)
    ///
    /// # Register our code on the sensor. When we call `VisionSensor.get_objects`, the associated
    /// # signatures will be returned as a single object if their colors are detected next to each
    /// # other.
    /// sensor.add_code(code)
    ///
    /// # Scan for objects.
    /// for object in sensor.get_objects():
    ///     # Filter only objects matching the code we just set.
    ///     if obj.source == DetectionSource.Code(code):
    ///         print("Detected object from code!")
    /// ```
    #[method]
    fn add_code(&self, code: &VisionCodeObj) -> Result<(), Exception> {
        self.guard.borrow_mut().add_code(code.code())?;
        Ok(())
    }

    /// Returns the user-set behavior of the LED indicator on the sensor.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there was not a sensor connected to the port, or if a read operation
    /// failed, or there was no signature previously set in the slot(s) specified in the
    /// `VisionCode`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// async def main():
    ///     # Set the LED to red at 100% brightness.
    ///     sensor.set_led_mode(LedMode.Manual(255, 0, 0, 1))
    ///
    ///     # Give the sensor time to update.
    ///     await vasyncio.Sleep(VisionSensor.UPDATE_INTERVAL_MS, MILLIS)
    ///
    ///     # Check the sensor's reported LED mode. Should be the same as what we just set.
    ///     led_mode = sensor.get_led_mode()
    ///     assert led_mode.r == 255
    ///     assert led_mode.g == 0
    ///     assert led_mode.b == 0
    ///     assert led_mode.brightness == 1
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    #[stub(sig = "(self) -> LedMode")]
    fn get_led_mode(&self) -> Result<Obj, Exception> {
        Ok(led_mode::new(self.guard.borrow().led_mode()?))
    }

    /// Returns a `tuple` of objects detected by the sensor.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there was not a sensor connected to the port, or if the sensor is in
    /// Wi-Fi mode, or if the sensor failed to read an object.
    ///
    /// # Examples
    ///
    /// With one signature:
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// # Set a color signature on the sensor's first slot.
    /// sensor.set_signature(1, VisionSignature(10049, 11513, 10781, -425, 1, -212, 4.1))
    ///
    /// # Scan for detected objects.
    /// for _ in sensor.get_objects():
    ///     print("Object detected")
    /// ```
    ///
    /// With multiple signatures:
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// # Two color signatures.
    /// sig_1 = VisionSignature(10049, 11513, 10781, -425, 1, -212, 4.1)
    /// sig_2 = VisionSignature(8973, 11143, 10058, -2119, -1053, -1586, 5.4)
    ///
    /// # Store the signatures on the sensor.
    ///
    /// sensor.set_signature(1, sig_1)
    /// sensor.set_signature(2, sig_2)
    ///
    /// # Scan for objects.
    /// for object in sensor.get_objects():
    ///     # Identify which signature the detected object matches.
    ///     if object.source == DetectionSource.Signature(1):
    ///         print("Detected object matching sig_1")
    ///     elif object.source == DetectionSource.Signature(2):
    ///         print("Detected object matching sig_2")
    /// ```
    #[method]
    #[stub(sig = "(self) -> tuple[VisionObject, ...]")]
    fn get_objects(&self) -> Result<Obj, Exception> {
        let objects = self.guard.borrow().objects()?;
        let obj_objects = objects
            .into_iter()
            .map(VisionObjectObj::new)
            .map(Obj::from)
            .collect::<Vec<_>>();
        Ok(new_tuple(&obj_objects))
    }

    /// Returns the number of objects detected by the sensor.
    ///
    /// # Raises
    ///
    /// - `DeviceError`: If there was not a sensor connected to the port, or if the sensor is in
    /// Wi-Fi mode, or if the sensor failed to read an object.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    ///
    /// async def main():
    ///     # Set a color signature on the sensor's first slot.
    ///     sensor.set_signature(1, VisionSignature(10049, 11513, 10781, -425, 1, -212, 4.1))
    ///     while True:
    ///         object_count = sensor.get_object_count()
    ///         print(f"Sensor is currently detecting {object_count} objects.")
    ///
    ///         await vasyncio.Sleep(VisionSensor.UPDATE_INTERVAL_MS, MILLIS)
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_object_count(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().object_count()? as i32)
    }

    /// Returns the current brightness setting of the vision sensor as a percentage.
    ///
    /// The returned result should be from `0.0` (0%) to `1.0` (100%).
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
    /// sensor = VisionSensor(1)
    ///
    /// async def main():
    ///     # Set brightness to 50%
    ///     sensor.set_brightness(0.5)
    ///
    ///     # Give the sensor time to update.
    ///     await vasyncio.Sleep(VisionSensor.UPDATE_INTERVAL_MS, MILLIS)
    ///
    ///     # Read brightness. Should be 50%, since we just set it.
    ///     assert sensor.get_brightness() == 0.5
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    fn get_brightness(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().brightness()? as f32)
    }

    /// Returns the current white balance of the vision sensor as an RGB color.
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
    /// async def main():
    ///     # Set white balance to manual.
    ///     sensor.set_white_balance(WhiteBalance.Manual(255, 255, 255))
    ///
    ///     # Give the sensor time to update.
    ///     await vasyncio.Sleep(VisionSensor.UPDATE_INTERVAL_MS, MILLIS)
    ///
    ///     # Read brightness. Should be 50%, since we just set it.
    ///     white_balance = sensor.get_balance()
    ///     assert isinstance(white_balance, WhiteBalance.Manual)
    ///     assert white_balance.r == 255
    ///     assert white_balance.g == 255
    ///     assert white_balance.b == 255
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    #[stub(sig = "(self) -> WhiteBalance")]
    fn get_white_balance(&self) -> Result<Obj, Exception> {
        Ok(white_balance::new(self.guard.borrow().white_balance()?))
    }

    /// Sets the brightness percentage of the vision sensor. Should be between 0.0 and 1.0.
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
    /// sensor = VisionSensor(1)
    ///
    /// # Set brightness to 50%
    /// sensor.set_brightness(0.5)
    /// ```
    #[method]
    fn set_brightness(&self, brightness: f32) -> Result<(), Exception> {
        self.guard.borrow_mut().set_brightness(brightness as f64)?;
        Ok(())
    }

    /// Sets the white balance of the vision sensor.
    ///
    /// White balance can be either automatically set or manually set through an RGB color.
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
    /// sensor = VisionSensor(1)
    ///
    /// # Set white balance to manual.
    /// sensor.set_white_balance(WhiteBalance.Manual(255, 255, 255))
    /// ```
    #[method]
    #[stub(sig = "(self, balance: WhiteBalance) -> None")]
    fn set_white_balance(&self, balance: WhiteBalanceArg) -> Result<(), Exception> {
        self.guard.borrow_mut().set_white_balance(balance.0)?;
        Ok(())
    }

    /// Configure the behavior of the LED indicator on the sensor.
    ///
    /// The default behavior is represented by `LedMode.Auto`, which will display the color of
    /// the most prominent detected object's signature color. Alternatively, the LED can be
    /// configured to display a single RGB color.
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
    /// sensor = VisionSensor(1)
    ///
    /// # Set the LED to red at 100% brightness.
    /// sensor.set_led_mode(LedMode.Manual(255, 0, 0, 1))
    /// ```
    #[method]
    #[stub(sig = "(self, mode: LedMode) -> None")]
    fn set_led_mode(&self, mode: LedModeArg) -> Result<(), Exception> {
        self.guard.borrow_mut().set_led_mode(mode.0)?;
        Ok(())
    }

    /// Sets the vision sensor's detection mode. See `VisionMode` for more information on what each
    /// mode does.
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
    /// sensor = VisionSensor(1)
    ///
    /// # Place the sensor into "Wi-Fi mode," allowing you to connect to it via a hotspot and
    /// # receive a video stream of its camera from another device.
    /// sensor.set_mode(VisionMode.WIFI)
    /// ```
    #[method]
    fn set_mode(&self, mode: &VisionModeObj) -> Result<(), Exception> {
        self.guard.borrow_mut().set_mode(mode.mode())?;
        Ok(())
    }

    /// Returns the current detection mode that the sensor is using.
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
    /// sensor = VisionSensor(1)
    ///
    /// async def main():
    ///     # Place the sensor into "Wi-Fi mode," allowing you to connect to it via a hotspot and
    ///     # receive a video stream of its camera from another device.
    ///     sensor.set_mode(VisionMode.WIFI)
    ///
    ///     await vasyncio.Sleep(VisionSensor.UPDATE_INTERVAL_MS, MILLIS)
    ///
    ///     # Since we just set the mode, we can the mode off the sensor to verify that it's now in
    ///     # Wi-Fi mode.
    ///     assert sensor.get_mode() == VisionMode.WIFI
    ///
    /// vasyncio.run(main)
    /// ```
    #[method]
    #[stub(sig = "(self) -> VisionMode")]
    fn get_mode(&self) -> Result<Obj, Exception> {
        Ok(Obj::from_static(match self.guard.borrow().mode()? {
            VisionMode::ColorDetection => VisionModeObj::COLOR_DETECTION,
            VisionMode::LineDetection => VisionModeObj::LINE_DETECTION,
            VisionMode::MixedDetection => VisionModeObj::MIXED_DETECTION,
            VisionMode::Wifi => VisionModeObj::WIFI,
            VisionMode::Test => VisionModeObj::TEST,
        }))
    }
}
