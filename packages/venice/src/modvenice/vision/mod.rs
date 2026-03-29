pub mod code;
pub mod led_mode;
pub mod mode;
pub mod object;
pub mod signature;
pub mod source;
pub mod white_balance;

use argparse::{ArgParser, Args, DefaultParser, IntParser, error_msg};
use micropython_rs::{
    class, class_methods,
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

#[class(qstr!(VisionSensor))]
pub struct VisionSensorObj {
    base: ObjBase,
    guard: SmartGuard<VisionSensor>,
}

#[class_methods]
impl VisionSensorObj {
    #[constant]
    const HORIZONTAL_RESOLUTION: i32 = VisionSensor::HORIZONTAL_RESOLUTION as i32;
    #[constant]
    const VERTICAL_RESOLUTION: i32 = VisionSensor::VERTICAL_RESOLUTION as i32;

    #[constant]
    const HORIZONTAL_FOV: f32 = VisionSensor::HORIZONTAL_FOV;
    #[constant]
    const VERTICAL_FOV: f32 = VisionSensor::VERTICAL_FOV;
    #[constant]
    const DIAGONAL_FOV: f32 = VisionSensor::DIAGONAL_FOV;

    #[constant]
    const UPDATE_INTERVAL_MS: i32 = VisionSensor::UPDATE_INTERVAL.as_millis() as i32;

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
        let guard = devices::lock_port(port, |p| VisionSensor::new(p));

        Ok(Self {
            base: ObjBase::new(ty),
            guard,
        })
    }

    #[method]
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

    #[method]
    fn get_signature(&self, id: SignatureId) -> Result<Option<VisionSignatureObj>, Exception> {
        Ok(self
            .guard
            .borrow()
            .signature(id.id())?
            .map(VisionSignatureObj::new))
    }

    #[method]
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

    #[method]
    fn add_code(&self, code: &VisionCodeObj) -> Result<(), Exception> {
        self.guard.borrow_mut().add_code(code.code())?;
        Ok(())
    }

    #[method]
    fn get_led_mode(&self) -> Result<Obj, Exception> {
        Ok(led_mode::new(self.guard.borrow().led_mode()?))
    }

    #[method]
    fn get_objects(&self) -> Result<Obj, Exception> {
        let objects = self.guard.borrow().objects()?;
        let obj_objects = objects
            .into_iter()
            .map(VisionObjectObj::new)
            .map(Obj::from)
            .collect::<Vec<_>>();
        Ok(new_tuple(&obj_objects))
    }

    #[method]
    fn get_object_count(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().object_count()? as i32)
    }

    #[method]
    fn get_brightness(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().brightness()? as f32)
    }

    #[method]
    fn get_white_balance(&self) -> Result<Obj, Exception> {
        Ok(white_balance::new(self.guard.borrow().white_balance()?))
    }

    #[method]
    fn set_brightness(&self, brightness: f32) -> Result<(), Exception> {
        self.guard.borrow_mut().set_brightness(brightness as f64)?;
        Ok(())
    }

    #[method]
    fn set_white_balance(&self, balance: WhiteBalanceArg) -> Result<(), Exception> {
        self.guard.borrow_mut().set_white_balance(balance.0)?;
        Ok(())
    }

    #[method]
    fn set_led_mode(&self, mode: LedModeArg) -> Result<(), Exception> {
        self.guard.borrow_mut().set_led_mode(mode.0)?;
        Ok(())
    }

    #[method]
    fn set_mode(&self, mode: &VisionModeObj) -> Result<(), Exception> {
        self.guard.borrow_mut().set_mode(mode.mode())?;
        Ok(())
    }

    #[method]
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
