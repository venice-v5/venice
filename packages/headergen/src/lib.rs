use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub const HEADERGEN_DIR_NAME: &str = "headergen";

pub fn collect_dir_files(
    dir: &Path,
    extension: &OsStr,
    recursive: bool,
    vec: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;

        let path = entry.path();
        let file_type = entry.file_type().expect("couldn't get file type");
        if recursive && file_type.is_dir() {
            collect_dir_files(&path, extension, true, vec)?;
        } else if file_type.is_file() && path.extension() == Some(extension) {
            vec.push(path);
        }
    }

    Ok(())
}

pub fn collect_c_srcs(
    mp_dir: &Path,
    py_dir: &Path,
    port_dir: &Path,
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut c_srcs = Vec::new();

    collect_dir_files(&py_dir, OsStr::new("c"), false, &mut c_srcs)?;
    collect_dir_files(&port_dir, OsStr::new("c"), false, &mut c_srcs)?;
    c_srcs.push(mp_dir.join("shared/readline/readline.c"));

    Ok(c_srcs)
}
