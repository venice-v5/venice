mod april_tag_family;
mod ai_vision_object;
mod ai_vision_flags;
mod ai_vision_detection_mode;
mod ai_vision_color;
use micropython_rs::obj::{ObjFullType, TypeFlags};

use crate::qstrgen::qstr;

static AI_VISION_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionSensor));
