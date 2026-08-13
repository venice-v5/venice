use std::ffi::c_void;

use crate::{
    except::{RUNTIME_ERROR_TYPE, raise_msg},
    init::token,
    nlr,
    obj::{Obj, ObjType},
};

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

    fn mp_obj_is_subclass_fast(object: Obj, classinfo: Obj) -> bool;

    static mp_type_gen_instance: ObjType;
    static mp_type_GeneratorExit: ObjType;
    static mp_const_GeneratorExit_obj: c_void;
}

pub const GEN_INSTANCE_TYPE: &ObjType = unsafe { &mp_type_gen_instance };

pub struct ResumeResult {
    pub obj: Obj,
    pub return_kind: VmReturnKind,
}

pub fn resume_gen(obj: Obj, send_val: Obj, throw_val: Obj) -> ResumeResult {
    let mut ret = Obj::NONE;
    let return_kind = unsafe { mp_obj_gen_resume(obj, send_val, throw_val, &raw mut ret) };
    ResumeResult {
        obj: ret,
        return_kind,
    }
}

/// Closes a generator by injecting `GeneratorExit` and running its synchronous cleanup.
///
/// This matches MicroPython's `generator.close()` behavior. A generator that yields during cleanup
/// raises `RuntimeError`, while exceptions other than `GeneratorExit` propagate to the caller.
pub fn close_gen(obj: Obj) {
    let generator_exit = unsafe {
        Obj::from_ptr(&raw const mp_const_GeneratorExit_obj as *const c_void as *mut c_void)
    };
    let result = resume_gen(obj, Obj::NONE, generator_exit);

    match result.return_kind {
        VmReturnKind::Normal => {}
        VmReturnKind::Yield => raise_msg(
            token(),
            RUNTIME_ERROR_TYPE,
            c"generator ignored GeneratorExit",
        ),
        VmReturnKind::Exception => {
            let exception_type =
                unsafe { Obj::from_ptr(result.obj.obj_type() as *const ObjType as *mut c_void) };
            let generator_exit_type = unsafe {
                Obj::from_ptr(&raw const mp_type_GeneratorExit as *const ObjType as *mut c_void)
            };

            if !unsafe { mp_obj_is_subclass_fast(exception_type, generator_exit_type) } {
                nlr::raise(token(), result.obj);
            }
        }
    }
}
