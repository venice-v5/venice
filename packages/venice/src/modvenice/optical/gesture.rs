use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_core::time::LowResolutionTime;
use vexide_devices::smart::optical::{Gesture, GestureDirection};

use crate::modvenice::{read_only_attr::read_only_attr, units::time::TimeUnitObj};

/// Gesture data from an `OpticalSensor`.
#[class(qstr!(Gesture))]
#[repr(C)]
pub struct GestureObj {
    base: ObjBase,
    direction: Obj,
    time: LowResolutionTime,
    count: u16,
    up: u8,
    down: u8,
    left: u8,
    right: u8,
    gesture_type: u8,
}

/// Represents a gesture and its direction.
#[class(qstr!(GestureDirection))]
#[repr(C)]
pub struct GestureDirectionObj {
    base: ObjBase,
    direction: GestureDirection,
}

#[class_methods]
impl GestureDirectionObj {
    const fn new(direction: GestureDirection) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            direction,
        }
    }

    /// Up gesture.
    #[constant]
    pub const UP: &Self = &Self::new(GestureDirection::Up);

    /// Down gesture.
    #[constant]
    pub const DOWN: &Self = &Self::new(GestureDirection::Down);

    /// Left gesture.
    #[constant]
    pub const LEFT: &Self = &Self::new(GestureDirection::Left);

    /// Right gesture.
    #[constant]
    pub const RIGHT: &Self = &Self::new(GestureDirection::Right);

    pub fn direction(&self) -> GestureDirection {
        self.direction
    }

    #[printer]
    fn printer(&self, printer: &mut Print, _kind: PrintKind) {
        printer.print(match self.direction() {
            GestureDirection::Up => "GestureDirection.UP",
            GestureDirection::Down => "GestureDirection.DOWN",
            GestureDirection::Left => "GestureDirection.LEFT",
            GestureDirection::Right => "GestureDirection.RIGHT",
        });
    }
}

impl GestureObj {
    pub fn new(gesture: Gesture) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            direction: Obj::from_static(match gesture.direction {
                GestureDirection::Up => GestureDirectionObj::UP,
                GestureDirection::Down => GestureDirectionObj::DOWN,
                GestureDirection::Left => GestureDirectionObj::LEFT,
                GestureDirection::Right => GestureDirectionObj::RIGHT,
            }),
            time: gesture.time,
            count: gesture.count,
            up: gesture.up,
            down: gesture.down,
            left: gesture.left,
            right: gesture.right,
            gesture_type: gesture.gesture_type,
        }
    }
}

#[class_methods]
impl GestureObj {
    #[attr]
    #[stub(attrs = [
        "direction: GestureDirection",
        "count: int",
        "up: int",
        "down: int",
        "left: int",
        "right: int",
        "gesture_type: int",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };

        result.return_value(match attr.as_str() {
            "direction" => self.direction,
            "count" => Obj::from_int(self.count as i32),
            "up" => Obj::from_int(self.up as i32),
            "down" => Obj::from_int(self.down as i32),
            "left" => Obj::from_int(self.left as i32),
            "right" => Obj::from_int(self.right as i32),
            "gesture_type" => Obj::from_int(self.gesture_type as i32),
            _ => return,
        })
    }

    #[method]
    fn get_time(&self, unit: &TimeUnitObj) -> f32 {
        unit.unit()
            .dur_to_float(self.time.duration_since(LowResolutionTime::EPOCH)) // hack to get a Duration out of a LowResolutionTime
    }
}
