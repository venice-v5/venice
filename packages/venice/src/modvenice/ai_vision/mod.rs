pub mod ai_vision_color;
pub mod ai_vision_color_code;
pub mod ai_vision_detection_mode;
pub mod ai_vision_flags;
pub mod ai_vision_object;
pub mod april_tag_family;
use micropython_rs::{
    const_dict,
    except::raise_value_error,
    init::token,
    list::new_list,
    make_new_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
};
use vexide_devices::smart::ai_vision::AiVisionSensor;

use crate::{
    args::Args,
    devices::{self, PortNumber},
    fun::{fun1, fun2, fun3},
    modvenice::{
        ai_vision::{
            ai_vision_color::AiVisionColorObj, ai_vision_color_code::AiVisionColorCodeObj,
            ai_vision_detection_mode::AiVisionDetectionModeObj, ai_vision_flags::AiVisionFlagsObj,
            ai_vision_object::AiVisionObjectObj, april_tag_family::AprilTagFamilyObj,
        },
        raise_device_error,
    },
    obj::alloc_obj,
    qstrgen::qstr,
    registry::RegistryGuard,
};

static AI_VISION_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionSensor))
    .set_make_new(make_new_from_fn!(ai_vision_sensor_make_new))
    .set_locals_dict(const_dict![
        qstr!(MAX_OBJECTS) => Obj::from_int(AiVisionSensor::MAX_OBJECTS as i32),
        qstr!(HORIZONTAL_RESOLUTION) => Obj::from_int(AiVisionSensor::HORIZONTAL_RESOLUTION as i32),
        qstr!(VERTICAL_RESOLUTION) => Obj::from_int(AiVisionSensor::VERTICAL_RESOLUTION as i32),
        qstr!(HORIZONTAL_FOV) => Obj::from_float(AiVisionSensor::HORIZONTAL_FOV),
        qstr!(VERTICAL_FOV) => Obj::from_float(AiVisionSensor::VERTICAL_FOV),
        qstr!(DIAGONAL_FOV) => Obj::from_float(AiVisionSensor::DIAGONAL_FOV),

        qstr!(temperature) => Obj::from_static(&fun1!(ai_vision_sensor_temperature, &AiVisionSensorObj)),
        qstr!(set_color_code) => Obj::from_static(&fun3!(ai_vision_sensor_set_color_code, &AiVisionSensorObj, i32, &AiVisionColorCodeObj)),
        qstr!(color_code) => Obj::from_static(&fun2!(ai_vision_sensor_color_code, &AiVisionSensorObj, i32)),
        qstr!(color) => Obj::from_static(&fun2!(ai_vision_sensor_color, &AiVisionSensorObj, i32)),
        qstr!(set_color) => Obj::from_static(&fun3!(ai_vision_sensor_set_color, &AiVisionSensorObj, i32, &AiVisionColorObj)),
        qstr!(set_detection_mode) => Obj::from_static(&fun2!(ai_vision_sensor_set_detection_mode, &AiVisionSensorObj, &AiVisionDetectionModeObj)),
        qstr!(flags) => Obj::from_static(&fun1!(ai_vision_sensor_flags, &AiVisionSensorObj)),
        qstr!(set_flags) => Obj::from_static(&fun2!(ai_vision_sensor_set_flags, &AiVisionSensorObj, &AiVisionFlagsObj)),
        qstr!(start_awb) => Obj::from_static(&fun1!(ai_vision_sensor_start_awb, &AiVisionSensorObj)),
        qstr!(enable_test) => Obj::from_static(&fun2!(ai_vision_sensor_enable_test, &AiVisionSensorObj, i32)),
        qstr!(set_apriltag_family) => Obj::from_static(&fun2!(ai_vision_sensor_set_apriltag_family, &AiVisionSensorObj, &AprilTagFamilyObj)),
        qstr!(object_count) => Obj::from_static(&fun1!(ai_vision_sensor_object_count, &AiVisionSensorObj)),
        qstr!(objects) => Obj::from_static(&fun1!(ai_vision_sensor_objects, &AiVisionSensorObj)),
        qstr!(color_codes) => Obj::from_static(&fun1!(ai_vision_sensor_color_codes, &AiVisionSensorObj)),
        qstr!(free) => Obj::from_static(&fun1!(ai_vision_sensor_free, &AiVisionSensorObj)),
    ]);

#[repr(C)]
pub struct AiVisionSensorObj {
    base: ObjBase<'static>,
    guard: RegistryGuard<'static, AiVisionSensor>,
}

unsafe impl ObjTrait for AiVisionSensorObj {
    const OBJ_TYPE: &ObjType = AI_VISION_OBJ_TYPE.as_obj_type();
}

fn ai_vision_sensor_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    let token = token().unwrap();
    let mut reader = Args::new(n_pos, n_kw, args).reader(token);
    reader.assert_npos(1, 1).assert_nkw(0, 0);

    let port = PortNumber::from_i32(reader.next_positional())
        .unwrap_or_else(|_| raise_value_error(token, "port number must be between 1 and 21"));

    let guard = devices::lock_port(port, |port| AiVisionSensor::new(port));

    alloc_obj(AiVisionSensorObj {
        base: ObjBase::new(ty),
        guard,
    })
}

fn ai_vision_sensor_temperature(this: &AiVisionSensorObj) -> Obj {
    let temp = this
        .guard
        .borrow()
        .temperature()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_float(temp as f32)
}

fn ai_vision_sensor_set_color_code(
    this: &AiVisionSensorObj,
    id: i32,
    code: &AiVisionColorCodeObj,
) -> Obj {
    this.guard
        .borrow_mut()
        .set_color_code(id as _, &code.code())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_color_code(this: &AiVisionSensorObj, id: i32) -> Obj {
    let code = this
        .guard
        .borrow()
        .color_code(id as _)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    if let Some(code) = code {
        alloc_obj(AiVisionColorCodeObj::new(code))
    } else {
        Obj::NONE
    }
}

fn ai_vision_sensor_set_color(this: &AiVisionSensorObj, id: i32, color: &AiVisionColorObj) -> Obj {
    this.guard
        .borrow_mut()
        .set_color(id as _, color.color())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_color(this: &AiVisionSensorObj, id: i32) -> Obj {
    let color = this
        .guard
        .borrow()
        .color(id as _)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    if let Some(color) = color {
        alloc_obj(AiVisionColorObj::new(color))
    } else {
        Obj::NONE
    }
}

fn ai_vision_sensor_set_detection_mode(
    this: &AiVisionSensorObj,
    mode: &AiVisionDetectionModeObj,
) -> Obj {
    this.guard
        .borrow_mut()
        .set_detection_mode(mode.mode())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_flags(this: &AiVisionSensorObj) -> Obj {
    let flags = this
        .guard
        .borrow()
        .flags()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    alloc_obj(AiVisionFlagsObj::new(flags))
}

fn ai_vision_sensor_set_flags(this: &AiVisionSensorObj, flags: &AiVisionFlagsObj) -> Obj {
    this.guard
        .borrow_mut()
        .set_flags(flags.flags())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_start_awb(this: &AiVisionSensorObj) -> Obj {
    this.guard
        .borrow_mut()
        .start_awb()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_enable_test(this: &AiVisionSensorObj, test: i32) -> Obj {
    this.guard
        .borrow_mut()
        .enable_test(test as u8)
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_set_apriltag_family(
    this: &AiVisionSensorObj,
    family: &AprilTagFamilyObj,
) -> Obj {
    this.guard
        .borrow_mut()
        .set_apriltag_family(family.family())
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::NONE
}

fn ai_vision_sensor_object_count(this: &AiVisionSensorObj) -> Obj {
    let count = this
        .guard
        .borrow()
        .object_count()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    Obj::from_int(count as i32)
}

fn ai_vision_sensor_objects(this: &AiVisionSensorObj) -> Obj {
    let objects = this
        .guard
        .borrow()
        .objects()
        .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")));
    let objects = objects
        .into_iter()
        .map(|obj| AiVisionObjectObj::create_obj(obj))
        .collect::<Vec<_>>();
    new_list(&objects[..])
}

fn ai_vision_sensor_color_codes(this: &AiVisionSensorObj) -> Obj {
    let this = this.guard.borrow();
    let codes = (0..7)
        .map(|n| {
            this.color_code(n)
                .unwrap_or_else(|e| raise_device_error(token().unwrap(), format!("{e}")))
        })
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

fn ai_vision_sensor_free(this: &AiVisionSensorObj) -> Obj {
    this.guard.free_or_raise();
    Obj::NONE
}
