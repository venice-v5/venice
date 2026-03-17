pub mod code;
pub mod led_mode;
pub mod mode;
pub mod object;
pub mod signature;
pub mod source;
pub mod white_balance;

use argparse::{ArgParser, Args, DefaultParser, IntParser};
use micropython_rs::{
    class, class_methods,
    except::raise_type_error,
    fun::Fun2,
    init::token,
    list::new_list,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::vision::{VisionMode, VisionSensor};

use crate::{
    devices::{self},
    modvenice::{
        Exception, raise_port_error,
        vision::{
            code::VisionCodeObj, mode::VisionModeObj, object::VisionObjectObj,
            signature::VisionSignatureObj,
        },
    },
    registry::RegistryGuard,
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

#[class(qstr!(VisionSensor))]
pub struct VisionSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, VisionSensor>,
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
        let mut reader = Args::new(n_pos, n_kw, args).reader(token());
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = reader.next_positional()?;
        let guard = devices::lock_port(port, |p| VisionSensor::new(p));

        Ok(Self {
            base: ObjBase::new(ty),
            guard,
        })
    }

    #[method]
    fn set_signature(&self, id: SignatureId, signature: &VisionSignatureObj) {
        self.guard
            .borrow_mut()
            .set_signature(id.id(), signature.signature())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_signature(&self, id: SignatureId) -> Option<VisionSignatureObj> {
        self.guard
            .borrow()
            .signature(id.id())
            .unwrap_or_else(|e| raise_port_error!(e))
            .map(VisionSignatureObj::new)
    }

    #[method]
    fn get_signatures(&self) -> Obj {
        let vec = self
            .guard
            .borrow()
            .signatures()
            .unwrap_or_else(|e| raise_port_error!(e))
            .into_iter()
            .map(|s| s.map(VisionSignatureObj::new))
            .map(Obj::from)
            .collect::<Vec<_>>();
        new_list(&vec)
    }

    #[method]
    fn add_code(&self, code: &VisionCodeObj) {
        self.guard
            .borrow_mut()
            .add_code(code.code())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_led_mode(&self) -> Obj {
        led_mode::new(
            self.guard
                .borrow()
                .led_mode()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn get_objects(&self) -> Obj {
        let objects = self
            .guard
            .borrow()
            .objects()
            .unwrap_or_else(|e| raise_port_error!(e));
        let obj_objects = objects
            .into_iter()
            .map(VisionObjectObj::new)
            .map(Obj::from)
            .collect::<Vec<_>>();
        new_list(&obj_objects)
    }

    #[method]
    fn get_object_count(&self) -> i32 {
        self.guard
            .borrow()
            .object_count()
            .unwrap_or_else(|e| raise_port_error!(e)) as i32
    }

    #[method]
    fn get_brightness(&self) -> f32 {
        self.guard
            .borrow()
            .brightness()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn get_white_balance(&self) -> Obj {
        white_balance::new(
            self.guard
                .borrow()
                .white_balance()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn set_brightness(&self, brightness: f32) {
        self.guard
            .borrow_mut()
            .set_brightness(brightness as f64)
            .unwrap_or_else(|e| raise_port_error!(e))
    }

    extern "C" fn set_white_balance(self_in: Obj, balance_obj: Obj) -> Obj {
        let balance = white_balance::from_obj(balance_obj).unwrap_or_else(|| {
            raise_type_error(token(), c"expected <WhiteBalance> for argument #1")
        });
        let this = self_in.try_as_obj::<Self>().unwrap();
        this.guard
            .borrow_mut()
            .set_white_balance(balance)
            .unwrap_or_else(|e| raise_port_error!(e));
        Obj::NONE
    }

    #[constant(qstr!(set_white_balance))]
    const SET_WHITE_BALANCE: &Fun2 = &Fun2::new(Self::set_white_balance);

    extern "C" fn set_led_mode(self_in: Obj, mode_obj: Obj) -> Obj {
        let mode = led_mode::from_obj(mode_obj)
            .unwrap_or_else(|| raise_type_error(token(), c"expected <LedMode> for argument #1"));

        let this = self_in.try_as_obj::<Self>().unwrap();
        this.guard
            .borrow_mut()
            .set_led_mode(mode)
            .unwrap_or_else(|e| raise_port_error!(e));
        Obj::NONE
    }

    #[constant(qstr!(set_led_mode))]
    const SET_LED_MODE: &Fun2 = &Fun2::new(Self::set_led_mode);

    #[method]
    fn set_mode(&self, mode: &VisionModeObj) {
        self.guard
            .borrow_mut()
            .set_mode(mode.mode())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn get_mode(&self) -> Obj {
        Obj::from_static(
            match self
                .guard
                .borrow()
                .mode()
                .unwrap_or_else(|e| raise_port_error!(e))
            {
                VisionMode::ColorDetection => VisionModeObj::COLOR_DETECTION,
                VisionMode::LineDetection => VisionModeObj::LINE_DETECTION,
                VisionMode::MixedDetection => VisionModeObj::MIXED_DETECTION,
                VisionMode::Wifi => VisionModeObj::WIFI,
                VisionMode::Test => VisionModeObj::TEST,
            },
        )
    }
}
