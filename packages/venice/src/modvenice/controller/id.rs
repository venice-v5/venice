use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::controller::ControllerId;

#[class(qstr!(ControllerId))]
#[repr(C)]
pub struct ControllerIdObj {
    base: ObjBase,
    id: ControllerId,
}

impl ControllerIdObj {
    const fn new(id: ControllerId) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            id,
        }
    }

    pub const fn id(&self) -> ControllerId {
        self.id
    }
}

#[class_methods]
impl ControllerIdObj {
    #[constant]
    pub const PRIMARY: &Self = &Self::new(ControllerId::Primary);
    #[constant]
    pub const PARTNER: &Self = &Self::new(ControllerId::Partner);

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.id {
            ControllerId::Primary => "ControllerId.PRIMARY",
            ControllerId::Partner => "ControllerId.PARTNER",
        })
    }
}
