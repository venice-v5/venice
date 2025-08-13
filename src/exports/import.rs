use alloc::vec::Vec;

use micropython_rs::{MicroPython, obj::Obj, qstr::Qstr};

pub fn absolute_name(mp: &MicroPython, mut level: i32, module_name: &[u8]) -> Vec<u8> {
    let qstr_obj = Obj::from_qstr(Qstr::from_bytes(b"__name__"));
    let current_module_name_obj = mp.globals_dict().map.get(qstr_obj).unwrap();
    let current_module_name = current_module_name_obj.get_str().unwrap();

    let mut p = current_module_name.len();
    while level > 0 && p != 0 {
        p -= 1;
        if module_name[p] == b'.' {
            level -= 1;
        }
    }

    let chopped_module_name = &current_module_name[p..];
    // add a byte for the dot
    let mut absolute_name = Vec::with_capacity(chopped_module_name.len() + module_name.len() + 1);
    absolute_name.extend_from_slice(chopped_module_name);
    absolute_name.push(b'.');
    absolute_name.extend_from_slice(module_name);

    absolute_name
}

pub fn import(mp: &mut MicroPython, module_name_obj: Obj, _fromtuple: Obj, level: i32) -> Obj {
    let module_name = module_name_obj
        .get_str()
        .expect("module name not a qstr or a str");

    if level != 0 {
        unimplemented!("relative imports not supported");
    }

    if module_name.is_empty() {
        // TODO: Add exception API
        panic!("module name cannot be empty");
    }

    let qstr = Qstr::from_bytes(module_name);

    let loaded_module = mp
        .state_ctx()
        .vm
        .mp_loaded_modules_dict
        .map
        .get(module_name_obj);
    if let Some(module) = loaded_module {
        return module;
    }

    let builtin = mp.builtin_module(qstr, false);
    if !builtin.is_null() {
        return builtin;
    }

    let bytecode = mp.module_map().get(module_name).expect("module not found");
    mp.exec_module(module_name_obj, *bytecode)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn venice_import(arg_count: usize, args: *const Obj) -> Obj {
    let args = unsafe { core::slice::from_raw_parts(args, arg_count) };

    let module_name_obj = args[0];
    let (fromtuple, level) = if args.len() >= 4 {
        let level = args[4].as_small_int();
        if level < 0 {
            // TODO: Add exception API
            panic!("level cannot be negative")
        } else {
            (args[3], level)
        }
    } else {
        (Obj::NONE, 0)
    };

    MicroPython::reenter(|mut mp| import(unsafe { mp.as_mut() }, module_name_obj, fromtuple, level))
        .expect("reentry failed")
}
