use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::smart::vision::VisionMode;

/// A possible "detection mode" for the Vision Sensor.
///
/// This class is root-importable. Pass one of its singleton values to `VisionSensor.set_mode`; the
/// current value is returned by `VisionSensor.get_mode`. `VisionMode` isn't constructed directly.
/// Values have readable representations such as `VisionMode.COLOR_DETECTION`.
#[class(qstr!(VisionMode))]
#[repr(C)]
pub struct VisionModeObj {
    base: ObjBase,
    mode: VisionMode,
}

#[class_methods]
impl VisionModeObj {
    const fn new(mode: VisionMode) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            mode,
        }
    }

    /// Uses color signatures and codes to identify objects in blocks.
    #[constant]
    pub const COLOR_DETECTION: &Self = &Self::new(VisionMode::ColorDetection);
    /// Uses line tracking to identify lines.
    #[constant]
    pub const LINE_DETECTION: &Self = &Self::new(VisionMode::LineDetection);
    /// Both color signatures and lines will be detected as objects.
    #[constant]
    pub const MIXED_DETECTION: &Self = &Self::new(VisionMode::MixedDetection);
    /// Sets the sensor into "Wi-Fi mode", which disables all forms of object detection and enables the
    /// sensor's onboard Wi-Fi hotspot for streaming camera data over a web server.
    ///
    /// Once enabled, the sensor will create a wireless network with an SSID in the format of `VISION_XXXX`.
    /// The sensor's camera feed is available at `192.168.1.1`.
    ///
    /// This mode will be automatically disabled when connected to field control.
    #[constant]
    pub const WIFI: &Self = &Self::new(VisionMode::Wifi);
    /// Unknown use.
    #[constant]
    pub const TEST: &Self = &Self::new(VisionMode::Test);

    pub fn mode(&self) -> VisionMode {
        self.mode
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.mode {
            VisionMode::ColorDetection => "VisionMode.COLOR_DETECTION",
            VisionMode::LineDetection => "VisionMode.LINE_DETECTION",
            VisionMode::MixedDetection => "VisionMode.MIXED_DETECTION",
            VisionMode::Wifi => "VisionMode.WIFI",
            VisionMode::Test => "VisionMode.TEST",
        });
    }
}
