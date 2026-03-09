pub mod ai_vision_color;
pub mod ai_vision_color_code;
pub mod ai_vision_detection_mode;
pub mod ai_vision_flags;
pub mod ai_vision_object;
pub mod april_tag_family;
use argparse::Args;
use micropython_rs::{
    class, class_methods,
    except::raise_value_error,
    init::token,
    list::new_list,
    obj::{Obj, ObjBase, ObjType},
};
use vexide_devices::smart::ai_vision::AiVisionSensor;

use crate::{
    devices::{self, PortNumber},
    modvenice::{
        ai_vision::{
            ai_vision_color::AiVisionColorObj, ai_vision_color_code::AiVisionColorCodeObj,
            ai_vision_detection_mode::AiVisionDetectionModeObj, ai_vision_flags::AiVisionFlagsObj,
            april_tag_family::AprilTagFamilyObj,
        },
        raise_port_error,
    },
    obj::alloc_obj,
    registry::RegistryGuard,
};

#[class(qstr!(AiVisionSensor))]
#[repr(C)]
pub struct AiVisionSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, AiVisionSensor>,
}

#[class_methods]
impl AiVisionSensorObj {
    #[make_new]
    fn make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Self {
        let token = token();
        let mut reader = Args::new(n_pos, n_kw, args).reader(token);
        reader.assert_npos(1, 1).assert_nkw(0, 0);

        let port = PortNumber::from_i32(reader.next_positional())
            .unwrap_or_else(|_| raise_value_error(token, c"port number must be between 1 and 21"));

        let guard = devices::lock_port(port, AiVisionSensor::new);

        Self {
            base: ObjBase::new(ty),
            guard,
        }
    }

    #[method]
    fn temperature(&self) -> f32 {
        self.guard
            .borrow()
            .temperature()
            .unwrap_or_else(|e| raise_port_error!(e)) as f32
    }

    #[method]
    fn set_color_code(&self, id: i32, code: &AiVisionColorCodeObj) {
        self.guard
            .borrow_mut()
            .set_color_code(id as _, &code.code())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn color_code(&self, id: i32) -> Option<AiVisionColorCodeObj> {
        self.guard
            .borrow()
            .color_code(id as _)
            .unwrap_or_else(|e| raise_port_error!(e))
            .map(AiVisionColorCodeObj::new)
    }

    #[method]
    fn set_color(&self, id: i32, color: &AiVisionColorObj) {
        self.guard
            .borrow_mut()
            .set_color(id as _, color.color())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn color(&self, id: i32) -> Option<AiVisionColorObj> {
        self.guard
            .borrow()
            .color(id as _)
            .unwrap_or_else(|e| raise_port_error!(e))
            .map(AiVisionColorObj::new)
    }

    #[method]
    fn set_detection_mode(&self, mode: &AiVisionDetectionModeObj) {
        self.guard
            .borrow_mut()
            .set_detection_mode(mode.mode())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn flags(&self) -> AiVisionFlagsObj {
        AiVisionFlagsObj::new(
            self.guard
                .borrow()
                .flags()
                .unwrap_or_else(|e| raise_port_error!(e)),
        )
    }

    #[method]
    fn set_flags(&self, flags: &AiVisionFlagsObj) {
        self.guard
            .borrow_mut()
            .set_flags(flags.flags())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn start_awb(&self) {
        self.guard
            .borrow_mut()
            .start_awb()
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn enable_test(&self, test: i32) {
        self.guard
            .borrow_mut()
            .enable_test(test as u8)
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn set_apriltag_family(&self, family: &AprilTagFamilyObj) {
        self.guard
            .borrow_mut()
            .set_apriltag_family(family.family())
            .unwrap_or_else(|e| raise_port_error!(e));
    }

    #[method]
    fn object_count(&self) -> i32 {
        let count = self
            .guard
            .borrow()
            .object_count()
            .unwrap_or_else(|e| raise_port_error!(e));
        count as i32
    }

    #[method]
    fn objects(&self) -> Obj {
        let objects = self
            .guard
            .borrow()
            .objects()
            .unwrap_or_else(|e| raise_port_error!(e));
        let objects = objects
            .into_iter()
            .map(ai_vision_object::create_obj)
            .collect::<Vec<_>>();
        new_list(&objects[..])
    }

    #[method]
    fn color_codes(&self) -> Obj {
        let guard = self.guard.borrow();
        let codes = (0..7)
            .map(|n| guard.color_code(n).unwrap_or_else(|e| raise_port_error!(e)))
            .map(|code| {
                if let Some(code) = code {
                    alloc_obj(AiVisionColorCodeObj::new(code))
                } else {
                    Obj::NONE
                }
            })
            .collect::<Vec<_>>();
        new_list(&codes[..])
    }

    #[method]
    fn free(&self) -> Obj {
        self.guard.free_or_raise();
        Obj::NONE
    }
}
