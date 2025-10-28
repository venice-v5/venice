use crate::obj::Obj;

#[repr(C)]
pub enum VmReturnKind {
    Normal,
    Yield,
    Exception,
}

unsafe extern "C" {
    fn mp_obj_gen_resume(
        self_in: Obj,
        send_val: Obj,
        throw_val: Obj,
        ret_val: *mut Obj,
    ) -> VmReturnKind;
}

pub struct ResumeResult {
    pub obj: Obj,
    pub return_kind: VmReturnKind,
}

pub fn resume_gen(obj: Obj, send_val: Obj, throw_val: Obj) -> ResumeResult {
    let mut ret = Obj::NONE;
    let return_kind = unsafe { mp_obj_gen_resume(obj, send_val, throw_val, &raw mut ret) };
    ResumeResult { obj, return_kind }
}
