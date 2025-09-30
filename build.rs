use core::writeln;
use std::{fs::OpenOptions, io::Write, path::Path, process::Command};

use format_bytes::write_bytes;
use regex::bytes::Regex;

fn generated_qstrs(out_dir: &Path) -> Vec<Vec<u8>> {
    let qstrdefs_generated_h = std::fs::read(out_dir.join("genhdr/qstrdefs.generated.h")).unwrap();

    let re0 = Regex::new(r#"QDEF0\(MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*), \d+, \d+, ".*"\)"#).unwrap();
    let re1 = Regex::new(r#"QDEF1\(MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*), \d+, \d+, ".*"\)"#).unwrap();

    let mut defs = Vec::new();

    for qdef0 in re0.captures_iter(&qstrdefs_generated_h) {
        defs.push(qdef0[1].to_vec());
    }

    for qdef1 in re1.captures_iter(&qstrdefs_generated_h) {
        defs.push(qdef1[1].to_vec());
    }

    defs
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir_path = Path::new(&out_dir);

    let mut open_ops = OpenOptions::new();
    open_ops.create(true).write(true).truncate(true);

    let libmpyv5 = std::env::var("VENICE_LIBMPYV5_PATH").unwrap_or_else(|_| {
        let output = Command::new("make")
            .arg(format!("BUILD={out_dir}"))
            .arg("-j")
            .current_dir(format!("{manifest_dir}/port"))
            .output()
            .expect("couldn't build micropython with `make`");
        if !output.status.success() {
            panic!(
                "couldn't build micropython: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        format!("{out_dir}/libmpyv5.a")
    });

    let generated_qstrs = generated_qstrs(out_dir_path);
    let mut generated_qstrs_rs = open_ops
        .open(out_dir_path.join("generated_qstrs.rs"))
        .unwrap();

    // must use writeln! because write_bytes! can't write single opening braces
    writeln!(
        generated_qstrs_rs,
        r"
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum GeneratedQstr {{
            MP_QSTRnull,
            MP_QSTR_,"
    )
    .unwrap();

    for generated_qstr in generated_qstrs.iter() {
        write_bytes!(&mut generated_qstrs_rs, b"MP_QSTR_{},\n", generated_qstr).unwrap();
    }

    writeln!(generated_qstrs_rs, "}}").unwrap();

    println!("cargo::rustc-env=GENERATED_QSTRS_RS={out_dir}/generated_qstrs.rs");

    println!("cargo::rustc-link-search=native={manifest_dir}/link");
    println!("cargo::rustc-link-lib=c");
    println!("cargo::rustc-link-lib=m");

    println!("cargo::rustc-link-arg={libmpyv5}");
}
