use std::borrow::Cow;

use micropython_rs::{
    except::{mp_type_ImportError, raise_msg, raise_value_error},
    init::{InitToken, token},
    module::{builtin_module, exec_module},
    obj::Obj,
    qstr::Qstr,
    state::{globals, loaded_modules},
};

use crate::{
    error_msg::error_msg,
    module_map::{MODULE_MAP, VptModuleFlags},
    qstrgen::qstr,
};

pub fn absolute_name(token: InitToken, mut level: i32, module_name: &str) -> String {
    const NAME_OBJ: Obj = Obj::from_qstr(qstr!(__name__));

    let current_module_name_obj = unsafe { (*globals(token)).map.get(NAME_OBJ).unwrap() };
    let current_module_name = current_module_name_obj.get_str().unwrap();

    let is_package = MODULE_MAP
        .get()
        .unwrap()
        .get(current_module_name.as_bytes())
        .unwrap()
        .flags()
        .contains(VptModuleFlags::IS_PACKAGE);
    if is_package {
        level -= 1;
    }

    let p = if level == 0 {
        current_module_name.len()
    } else {
        current_module_name
            .rmatch_indices('.') // Iterates backwards over dots, yielding (index, ".")
            .nth((level as usize) - 1) // Skips to the Nth dot (0-indexed)
            .map(|(i, _)| i) // Extracts just the byte index
            .unwrap_or(0) // If there aren't enough dots, defaults to 0
    };

    let chopped_module_name = &current_module_name[..p];
    // allocate and add a byte for the dot
    let mut absolute_name =
        String::with_capacity(chopped_module_name.len() + module_name.len() + 1);
    absolute_name.push_str(chopped_module_name);
    absolute_name.push('.');
    absolute_name.push_str(module_name);

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

    if let Some(module) = MODULE_MAP.get().unwrap().get(full_name.as_str().as_bytes()) {
        exec_module(token, full_name, module.payload())
    } else {
        raise_msg(
            token,
            &mp_type_ImportError,
            error_msg!("no module named '{}'", full_name.as_str()),
        );
    }
}

pub fn import(token: InitToken, module_name_qstr: Qstr, _fromtuple: Obj, level: i32) -> Obj {
    let mut module_name = Cow::Borrowed(module_name_qstr.as_str());

    if level != 0 {
        module_name = Cow::Owned(absolute_name(token, level, &module_name));
    }

    if module_name.is_empty() {
        // TODO: Add exception API
        raise_value_error(token, c"module name cannot be empty");
    }

    let mut outer_module_obj = Obj::NULL;

    let mut current_len = 0;
    for level_str in module_name.split('.') {
        current_len += level_str.len();

        let full_name = Qstr::from_str(&module_name[..current_len]);
        let level_name = Qstr::from_str(level_str);

        outer_module_obj = process_import_at_level(token, full_name, level_name, outer_module_obj);

        // Step over the dot for the next iteration's full_name calculation
        current_len += 1;
    }

    outer_module_obj
}

#[unsafe(no_mangle)]
unsafe extern "C" fn venice_import(arg_count: usize, args: *const Obj) -> Obj {
    let args = unsafe { core::slice::from_raw_parts(args, arg_count) };
    let token = token();

    let module_name_obj = args[0];
    let (fromtuple, level) = if args.len() >= 4 {
        let level = args[4].try_to_int().unwrap();
        if level < 0 {
            // TODO: Add exception API
            raise_value_error(token, c"level cannot be negative");
        } else {
            (args[3], level)
        }
    } else {
        (Obj::NONE, 0)
    };

    import(
        token,
        Qstr::from_str(module_name_obj.get_str().unwrap()),
        fromtuple,
        level,
    )
}
