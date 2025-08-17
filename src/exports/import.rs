use alloc::{borrow::Cow, vec::Vec};

use micropython_rs::{MicroPython, module::VptModuleFlags, obj::Obj, qstr::Qstr};

use crate::println;

pub fn absolute_name(mp: &MicroPython, mut level: i32, module_name: &[u8]) -> Vec<u8> {
    let qstr_obj = Obj::from_qstr(Qstr::from_bytes(b"__name__"));
    let current_module_name_obj = mp.globals().map.get(qstr_obj).unwrap();
    let current_module_name = current_module_name_obj.get_str().unwrap();

    let is_package = mp
        .module_map()
        .get(current_module_name)
        .unwrap()
        .flags()
        .contains(VptModuleFlags::IS_PACKAGE);
    if is_package {
        level -= 1;
    }

    let mut p = current_module_name.len();
    while level > 0 && p != 0 {
        p -= 1;
        if current_module_name[p] == b'.' {
            level -= 1;
        }
    }

    let chopped_module_name = &current_module_name[..p];
    // allocate and add a byte for the dot
    let mut absolute_name = Vec::with_capacity(chopped_module_name.len() + module_name.len() + 1);
    absolute_name.extend_from_slice(chopped_module_name);
    absolute_name.push(b'.');
    absolute_name.extend_from_slice(module_name);

    println!("absolute name: {}", str::from_utf8(&absolute_name).unwrap());

    absolute_name
}

pub fn process_import_at_level(
    mp: &mut MicroPython,
    full_name: Qstr,
    level_name: Qstr,
    outer_module_obj: Obj,
) -> Obj {
    if let Some(loaded) = mp
        .state_ctx()
        .vm
        .mp_loaded_modules_dict
        .map
        .get(Obj::from_qstr(full_name))
    {
        return loaded;
    }

    if outer_module_obj.is_null() {
        let builtin = mp.builtin_module(level_name, false);
        if !builtin.is_null() {
            return builtin;
        }
    }

    if let Some(module) = mp.module_map().get(full_name.bytes()) {
        mp.exec_module(full_name, module.payload())
    } else {
        panic!(
            "module {} not found",
            str::from_utf8(full_name.bytes()).unwrap_or("<invalid utf8 module name>")
        )
    }
}

pub fn import(mp: &mut MicroPython, module_name_qstr: Qstr, _fromtuple: Obj, level: i32) -> Obj {
    let mut module_name = Cow::Borrowed(module_name_qstr.bytes());

    if level != 0 {
        module_name = Cow::Owned(absolute_name(mp, level, &module_name));
    }

    if module_name.is_empty() {
        // TODO: Add exception API
        panic!("module name cannot be empty");
    }

    let mut outer_module_obj = Obj::NULL;
    let mut last_name = 0;

    for (mut i, &c) in module_name.iter().enumerate() {
        if c == b'.' || i == module_name.len() - 1 {
            if c != b'.' {
                i += 1;
            }

            let full_name = Qstr::from_bytes(&module_name[..i]);
            let level_name = Qstr::from_bytes(&module_name[last_name..i]);

            let module_obj = process_import_at_level(mp, full_name, level_name, outer_module_obj);
            outer_module_obj = module_obj;

            last_name = i + 1;
        }
    }

    outer_module_obj
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

    MicroPython::reenter(|mut mp| {
        import(
            unsafe { mp.as_mut() },
            Qstr::from_bytes(module_name_obj.get_str().unwrap()),
            fromtuple,
            level,
        )
    })
    .expect("reentry failed")
}
