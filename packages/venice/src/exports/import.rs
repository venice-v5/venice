use std::borrow::Cow;

use argparse::error_msg;
use micropython_rs::{
    except::{IMPORT_ERROR_TYPE, raise_msg, type_error, value_error},
    init::{InitToken, token},
    map::Dict,
    module::{Module, builtin_module, exec_module},
    nlr::push_nlr_callback,
    obj::Obj,
    qstr::Qstr,
    state::{globals, loaded_modules},
};
use venice_program_table::ProgramFlags;

use crate::module_map::MODULE_MAP;

fn resolve_relative_name(
    current_module_name: &str,
    is_package: bool,
    mut level: i32,
    module_name: &str,
) -> Option<String> {
    if is_package {
        level -= 1;
    }

    let prefix_len = if level == 0 {
        current_module_name.len()
    } else {
        current_module_name
            .rmatch_indices('.')
            .nth((level as usize) - 1)
            .map(|(index, _)| index)?
    };
    let prefix = &current_module_name[..prefix_len];

    let mut absolute_name = String::with_capacity(
        prefix.len() + module_name.len() + usize::from(!module_name.is_empty()),
    );
    absolute_name.push_str(prefix);
    if !module_name.is_empty() {
        absolute_name.push('.');
        absolute_name.push_str(module_name);
    }
    Some(absolute_name)
}

pub fn absolute_name(token: InitToken, globals_obj: Obj, level: i32, module_name: &str) -> String {
    const NAME_OBJ: Obj = Obj::from_qstr(qstr!(__name__));
    const PATH_OBJ: Obj = Obj::from_qstr(qstr!(__path__));

    let active_globals = globals(token);
    let is_active_globals = globals_obj.is_none() || globals_obj.inner() == active_globals.cast();
    let globals = if globals_obj.is_none() {
        unsafe { &*active_globals }
    } else {
        globals_obj
            .try_as_obj::<Dict>()
            .unwrap_or_else(|| type_error(c"globals must be a dict").raise(token))
    };
    let current_module_name_obj = globals
        .map
        .get(NAME_OBJ)
        .unwrap_or_else(|| type_error(c"globals must contain __name__").raise(token));
    let current_module_name = current_module_name_obj
        .get_str()
        .unwrap_or_else(|| type_error(c"globals __name__ must be a string").raise(token));

    let is_package = globals.map.get(PATH_OBJ).is_some()
        || is_active_globals
            && MODULE_MAP
                .get()
                .unwrap()
                .get(current_module_name.as_bytes())
                .and_then(|module| module.flags().ok())
                .unwrap_or(ProgramFlags::empty())
                .contains(ProgramFlags::IS_PACKAGE);

    resolve_relative_name(current_module_name, is_package, level, module_name)
        .unwrap_or_else(|| raise_msg(token, IMPORT_ERROR_TYPE, c"can't perform relative import"))
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
        return push_nlr_callback(
            token,
            || exec_module(token, full_name, module.payload),
            || {
                unsafe { &mut *loaded_modules(token) }
                    .map
                    .remove(Obj::from_qstr(full_name));
            },
            false,
        );
    }

    if outer_module_obj.is_null() {
        let extensible = builtin_module(token, level_name, true);
        if !extensible.is_null() {
            return extensible;
        }
    }

    raise_msg(
        token,
        IMPORT_ERROR_TYPE,
        error_msg!("no module named '{}'", full_name.as_str()),
    );
}

pub fn import(
    token: InitToken,
    module_name_qstr: Qstr,
    globals_obj: Obj,
    fromtuple: Obj,
    level: i32,
) -> Obj {
    let mut module_name = Cow::Borrowed(module_name_qstr.as_str());

    if level != 0 {
        module_name = Cow::Owned(absolute_name(token, globals_obj, level, &module_name));
    }

    if module_name.is_empty() {
        value_error(c"module name cannot be empty").raise(token);
    }

    let mut top_module_obj = Obj::NULL;
    let mut outer_module_obj = Obj::NULL;

    let mut current_len = 0;
    for level_str in module_name.split('.') {
        current_len += level_str.len();

        let full_name = Qstr::from_str(&module_name[..current_len]);
        let level_name = Qstr::from_str(level_str);
        let parent_module_obj = outer_module_obj;

        outer_module_obj = process_import_at_level(token, full_name, level_name, parent_module_obj);
        if !parent_module_obj.is_null() {
            parent_module_obj.store_attr(level_name, outer_module_obj);
        }
        if top_module_obj.is_null() {
            top_module_obj = outer_module_obj;
        }

        // Step over the dot for the next iteration's full_name calculation
        current_len += 1;
    }

    if !fromtuple.is_none() {
        import_missing_fromlist_submodules(token, &module_name, outer_module_obj, fromtuple);
        outer_module_obj
    } else {
        top_module_obj
    }
}

fn import_missing_fromlist_submodules(
    token: InitToken,
    module_name: &str,
    module_obj: Obj,
    fromtuple: Obj,
) {
    if module_obj.try_as_obj::<Module>().is_none() {
        return;
    }
    let Some(items) = fromtuple.try_array() else {
        return;
    };
    for item in items {
        let Some(name) = item.get_str() else {
            continue;
        };
        if name == "*" {
            continue;
        }

        let level_name = Qstr::from_str(name);
        let already_loaded = module_obj
            .try_as_obj::<Module>()
            .unwrap()
            .globals()
            .map
            .get(Obj::from_qstr(level_name))
            .is_some();
        if already_loaded {
            continue;
        }

        let mut full_name = String::with_capacity(module_name.len() + name.len() + 1);
        full_name.push_str(module_name);
        full_name.push('.');
        full_name.push_str(name);
        let Some(_) = MODULE_MAP.get().unwrap().get(full_name.as_bytes()) else {
            continue;
        };

        let child =
            process_import_at_level(token, Qstr::from_str(&full_name), level_name, module_obj);
        module_obj.store_attr(level_name, child);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn venice_import(arg_count: usize, args: *const Obj) -> Obj {
    let args = unsafe { core::slice::from_raw_parts(args, arg_count) };
    let token = token();

    let module_name_obj = args[0];
    let globals_obj = args.get(1).copied().unwrap_or(Obj::NONE);
    let (fromtuple, level) = if args.len() >= 5 {
        let level = args[4].try_to_int().unwrap();
        if level < 0 {
            value_error(c"level cannot be negative").raise(token);
        } else {
            (args[3], level)
        }
    } else if args.len() >= 4 {
        (args[3], 0)
    } else {
        (Obj::NONE, 0)
    };

    import(
        token,
        Qstr::from_str(module_name_obj.get_str().unwrap()),
        globals_obj,
        fromtuple,
        level,
    )
}
