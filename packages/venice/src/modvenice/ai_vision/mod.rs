pub mod ai_vision_color;
pub mod ai_vision_color_code;
pub mod ai_vision_detection_mode;
pub mod ai_vision_flags;
pub mod ai_vision_object;
pub mod april_tag_family;

use argparse::{Args, error_msg};
use micropython_rs::{
    class, class_methods,
    list::new_list,
    obj::{Obj, ObjBase, ObjType},
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
            ai_vision_color::AiVisionColorObj, ai_vision_color_code::AiVisionColorCodeObj,
            ai_vision_detection_mode::AiVisionDetectionModeObj, ai_vision_flags::AiVisionFlagsObj,
            april_tag_family::AprilTagFamilyObj,
        },
        device_error,
    },
    registry::SmartGuard,
};

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

        let guard = devices::lock_port(port, AiVisionSensor::new);

        Ok(Self {
            base: ObjBase::new(ty),
            guard,
        })
    }

    #[method]
    fn get_temperature(&self) -> Result<f32, Exception> {
        Ok(self.guard.borrow().temperature()? as f32)
    }

    #[method]
    fn set_color_code(&self, id: i32, code: &AiVisionColorCodeObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_color_code(id as _, &code.code())?)
    }

    #[method]
    fn get_color_code(&self, id: i32) -> Result<Option<AiVisionColorCodeObj>, Exception> {
        Ok(self
            .guard
            .borrow()
            .color_code(id as _)?
            .map(AiVisionColorCodeObj::new))
    }

    #[method]
    fn set_color(&self, id: i32, color: &AiVisionColorObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_color(id as _, color.color())?)
    }

    #[method]
    fn get_color(&self, id: i32) -> Result<Option<AiVisionColorObj>, Exception> {
        Ok(self
            .guard
            .borrow()
            .color(id as _)?
            .map(AiVisionColorObj::new))
    }

    #[method]
    fn set_detection_mode(&self, mode: &AiVisionDetectionModeObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_detection_mode(mode.mode())?)
    }

    #[method]
    fn get_flags(&self) -> Result<AiVisionFlagsObj, Exception> {
        Ok(AiVisionFlagsObj::new(self.guard.borrow().flags()?))
    }

    #[method]
    fn set_flags(&self, flags: &AiVisionFlagsObj) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().set_flags(flags.flags())?)
    }

    #[method]
    fn start_awb(&self) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().start_awb()?)
    }

    #[method]
    fn enable_test(&self, test: i32) -> Result<(), Exception> {
        Ok(self.guard.borrow_mut().enable_test(test as u8)?)
    }

    #[method]
    fn set_apriltag_family(&self, family: &AprilTagFamilyObj) -> Result<(), Exception> {
        Ok(self
            .guard
            .borrow_mut()
            .set_apriltag_family(family.family())?)
    }

    #[method]
    fn get_object_count(&self) -> Result<i32, Exception> {
        Ok(self.guard.borrow().object_count()? as i32)
    }

    #[method]
    fn get_objects(&self) -> Result<Obj, Exception> {
        let objects = self.guard.borrow().objects()?;
        let objects = objects
            .into_iter()
            .map(ai_vision_object::create_obj)
            .collect::<Vec<_>>();
        Ok(new_list(&objects[..]))
    }

    #[method]
    fn get_color_codes(&self) -> Result<Obj, Exception> {
        let guard = self.guard.borrow();
        let codes = (0..7)
            .map(|n| guard.color_code(n))
            .map(|code| code.map(|code| Obj::from(code.map(AiVisionColorCodeObj::new))))
            .collect::<Result<Vec<_>, PortError>>()?;
        Ok(new_list(&codes[..]))
    }

    #[method]
    fn free(&self) -> Obj {
        self.guard.free_or_raise();
        Obj::NONE
    }
}
