use std::borrow::Cow;

use micropython_rs::{
    init::{InitToken, token},
    module::{builtin_module, exec_module},
    obj::Obj,
    qstr::Qstr,
    state::{globals, loaded_modules},
};

use crate::{
    module_map::{MODULE_MAP, VptModuleFlags},
    qstrgen::qstr,
};

pub fn absolute_name(token: InitToken, mut level: i32, module_name: &[u8]) -> Vec<u8> {
    const NAME_OBJ: Obj = Obj::from_qstr(qstr!(__name__));

    let current_module_name_obj = unsafe { (*globals(token)).map.get(NAME_OBJ).unwrap() };
    let current_module_name = current_module_name_obj.get_str().unwrap();

    let is_package = MODULE_MAP
        .get()
        .unwrap()
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

    absolute_name
}

pub fn process_import_at_level(
    token: InitToken,
    full_name: Qstr,
    level_name: Qstr,
    outer_module_obj: Obj,
) -> Obj {
    if let Some(loaded) = unsafe { (*loaded_modules(token)).map.get(Obj::from_qstr(full_name)) } {
        return loaded;
    }

    if outer_module_obj.is_null() {
        let builtin = builtin_module(token, level_name, false);
        if !builtin.is_null() {
            return builtin;
        }
    }

    if let Some(module) = MODULE_MAP.get().unwrap().get(full_name.bytes()) {
        exec_module(token, full_name, module.payload())
    } else {
        panic!(
            "module {} not found",
            str::from_utf8(full_name.bytes()).unwrap_or("<invalid utf8 module name>")
        )
    }
}

pub fn import(token: InitToken, module_name_qstr: Qstr, _fromtuple: Obj, level: i32) -> Obj {
    let mut module_name = Cow::Borrowed(module_name_qstr.bytes());

    if level != 0 {
        module_name = Cow::Owned(absolute_name(token, level, &module_name));
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

            let module_obj =
                process_import_at_level(token, full_name, level_name, outer_module_obj);
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
        let level = args[4].as_small_int().unwrap();
        if level < 0 {
            // TODO: Add exception API
            panic!("level cannot be negative")
        } else {
            (args[3], level)
        }
    } else {
        (Obj::NONE, 0)
    };

    import(
        token().unwrap(),
        Qstr::from_bytes(module_name_obj.get_str().unwrap()),
        fromtuple,
        level,
    )
}
