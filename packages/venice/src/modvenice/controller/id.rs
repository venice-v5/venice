use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
};
use vexide_devices::controller::ControllerId;


#[class(qstr!(ControllerId))]
#[repr(C)]
pub struct ControllerIdObj {
    base: ObjBase<'static>,
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
}
