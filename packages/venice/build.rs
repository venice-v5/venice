use core::writeln;
use std::{
    ffi::OsStr,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use regex::bytes::Regex;

fn collect_dir_files(dir: &Path, extension: &OsStr, recursive: bool, vec: &mut Vec<PathBuf>) {
    std::fs::read_dir(&dir)
        .expect("couldn't ls directory")
        .map(|entry| entry.expect("couldn't ls directory"))
        .for_each(|entry| {
            let path = entry.path();
            let file_type = entry.file_type().expect("couldn't get file type");
            if recursive && file_type.is_dir() {
                collect_dir_files(&path, extension, true, vec);
            } else if file_type.is_file() && path.extension() == Some(extension) {
                vec.push(path);
            }
        });
}

struct Builder {
    out_dir: String,
    mp_dir: String,
    py_dir: String,
    port_dir: String,
    genhdr_dir: String,
    c_srcs: Vec<PathBuf>,
    rust_srcs: Vec<PathBuf>,
}

impl Builder {
    fn new(manifest_dir: &str) -> Self {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let mp_dir = format!("{manifest_dir}/micropython");
        let py_dir = format!("{mp_dir}/py");
        let port_dir = format!("{manifest_dir}/port");

        let mut c_srcs = Vec::new();
        collect_dir_files(Path::new(&py_dir), OsStr::new("c"), false, &mut c_srcs);
        collect_dir_files(Path::new(&port_dir), OsStr::new("c"), false, &mut c_srcs);

        let mut rust_srcs = Vec::new();
        collect_dir_files(
            Path::new(&format!("{manifest_dir}/src")),
            OsStr::new("rs"),
            true,
            &mut rust_srcs,
        );

        let genhdr_dir = format!("{out_dir}/genhdr");
        std::fs::create_dir_all(&genhdr_dir).expect("couldn't create genhdr dir");

        Builder {
            py_dir: format!("{mp_dir}/py"),
            out_dir,
            mp_dir,
            port_dir,
            genhdr_dir,
            c_srcs,
            rust_srcs,
        }
    }

    fn gen_version_header(&self) {
        Command::new("python3")
            .arg(format!("{}/makeversionhdr.py", self.py_dir))
            .arg(format!("{}/mpversion.h", self.genhdr_dir))
            .status()
            .expect("couldn't generate mp version header");
    }

    fn gen_qstrdefs(&self, qstrs: &[Vec<u8>]) {
        let qstrdefs_file_path = format!("{}/qstrdefs.preprocessed.h", self.genhdr_dir);
        let mut qstrdefs_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&qstrdefs_file_path)
            .expect("couldn't open qstrdefs file");

        const BYTES_IN_LEN: usize = 1;
        const BYTES_IN_HASH: usize = 2;

        writeln!(
            &mut qstrdefs_file,
            "QCFG(BYTES_IN_LEN, ({BYTES_IN_LEN}))
            QCFG(BYTES_IN_HASH, ({BYTES_IN_HASH}))"
        )
        .expect("couldn't write to qstrdefs file");

        for qstr in qstrs.iter() {
            write!(
                &mut qstrdefs_file,
                "Q({})\n",
                str::from_utf8(qstr).expect("non-utf8 qstr")
            )
            .expect("couldn't write to qstrdefs file");
        }

        let generated_qstrs = Command::new("python3")
            .arg(format!("{}/makeqstrdata.py", self.py_dir))
            .arg(&qstrdefs_file_path)
            .output()
            .expect("coulnd't process qstr data")
            .stdout;

        std::fs::write(
            format!("{}/qstrdefs.generated.h", self.genhdr_dir),
            generated_qstrs,
        )
        .expect("couldn't write out qstr data");
    }

    fn gen_moduledefs(&self, moduledefs: &[Vec<u8>]) {
        let moduledefs_collected_path = format!("{}/moduledefs.collected", self.genhdr_dir);
        let mut moduledefs_collected = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&moduledefs_collected_path)
            .expect("couldn't open moduledefs file");

        for moduledef in moduledefs.iter() {
            writeln!(
                &mut moduledefs_collected,
                "{}",
                str::from_utf8(moduledef).expect("non-utf8 moduledef")
            )
            .expect("couldn't write to moduledefs file");
        }

        let moduledefs_h = Command::new("python3")
            .arg(format!("{}/makemoduledefs.py", self.py_dir))
            .arg(&moduledefs_collected_path)
            .output()
            .expect("couldn't generate moduledefs")
            .stdout;

        std::fs::write(format!("{}/moduledefs.h", self.genhdr_dir), &moduledefs_h)
            .expect("couldn't write out moduledefs");
    }

    fn gen_root_pointers(&self, root_pointers: &[Vec<u8>]) {
        let mut root_pointers_h = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{}/root_pointers.h", self.genhdr_dir))
            .expect("couldn't open root pointers file");

        for root_pointer in root_pointers.iter() {
            writeln!(
                &mut root_pointers_h,
                "{};",
                str::from_utf8(root_pointer).expect("non-utf8 root pointer")
            )
            .expect("couldn't write out root pointer");
        }
    }

    fn gen_headers(&self) {
        let mut qstrs = Vec::new();
        let mut moduledefs = Vec::new();
        let mut root_pointers = Vec::new();

        let c_qstr_re = Regex::new(r#"MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap();
        let c_moduledef_re = Regex::new(
            r#"(?:MP_REGISTER_MODULE|MP_REGISTER_EXTENSIBLE_MODULE|MP_REGISTER_MODULE_DELEGATION)\(.*?,\s*.*?\);"#,
        ).unwrap();
        let c_root_pointer_re = Regex::new(r#"MP_REGISTER_ROOT_POINTER\((.*?)\);"#).unwrap();

        let config_headers = [
            PathBuf::from(format!("{}/mpconfig.h", self.mp_dir)),
            PathBuf::from(format!("{}/mpconfigport.h", self.port_dir)),
        ];
        let c_qstr_src = self.c_srcs.iter().chain(config_headers.iter());

        for c_src in c_qstr_src {
            // should be replaced by clang -E
            let out = Command::new("arm-none-eabi-cpp")
                .arg("-I")
                .arg(&self.port_dir)
                .arg("-I")
                .arg(&self.mp_dir)
                .arg("-I")
                .arg(&self.out_dir)
                .arg("-DNO_QSTR")
                .arg(c_src)
                .output()
                .expect("couldn't preprocess C code")
                .stdout;

            for qstr_cap in c_qstr_re.captures_iter(&out) {
                qstrs.push(qstr_cap[1].to_vec());
            }

            for moduledef_cap in c_moduledef_re.captures_iter(&out) {
                moduledefs.push(moduledef_cap[0].to_vec());
            }

            for root_pointer_cap in c_root_pointer_re.captures_iter(&out) {
                root_pointers.push(root_pointer_cap[1].to_vec());
            }
        }

        let rust_qstr_re = Regex::new(r#"qstr!\(([a-zA-Z_][a-zA-Z0-9_]*)\)"#).unwrap();
        for rust_src in self.rust_srcs.iter() {
            let out = std::fs::read(rust_src).expect("couldn't read rust source");

            for cap in rust_qstr_re.captures_iter(&out) {
                qstrs.push(cap[1].to_vec());
            }
        }

        self.gen_qstrdefs(&qstrs);
        self.gen_moduledefs(&moduledefs);
        self.gen_root_pointers(&root_pointers);
    }

    fn gen_qstrs_rs(&self) {
        let qstrdefs_generated_h =
            std::fs::read_to_string(format!("{}/qstrdefs.generated.h", self.genhdr_dir))
                .expect("couldn't read generated qstrdefs");

        let qdef0_re =
            regex::Regex::new(r#"QDEF0\(MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*), \d+, \d+, ".*"\)"#)
                .unwrap();
        let qdef1_re =
            regex::Regex::new(r#"QDEF1\(MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*), \d+, \d+, ".*"\)"#)
                .unwrap();

        let mut defs = Vec::new();
        for qdef0_cap in qdef0_re.captures_iter(&qstrdefs_generated_h) {
            defs.push(qdef0_cap[1].to_string());
        }

        for qdef1_cap in qdef1_re.captures_iter(&qstrdefs_generated_h) {
            defs.push(qdef1_cap[1].to_string());
        }

        let generated_qstrs_rs_path = format!("{}/generated_qstrs.rs", self.out_dir);
        let mut generated_qstrs_rs = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&generated_qstrs_rs_path)
            .expect("couldn't open generated_qstrs.rs file");

        writeln!(
            &mut generated_qstrs_rs,
            r"
            #[allow(non_camel_case_types, dead_code)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            #[repr(C)]
            pub enum GeneratedQstr {{
                MP_QSTRnull,
                MP_QSTR_,
            "
        )
        .expect("couldn't write out generated qstrs");

        for def in defs.iter() {
            writeln!(&mut generated_qstrs_rs, "    MP_QSTR_{},\n", def)
                .expect("couldn't write out generated qstrs");
        }

        writeln!(&mut generated_qstrs_rs, "}}").expect("couldn't write oout generated qstrs");

        println!(
            "cargo::rustc-env=GENERATED_QSTRS_RS={}",
            generated_qstrs_rs_path
        );
    }

    fn compile_mp(&self) {
        let mut build = cc::Build::new();
        build
            .files(&self.c_srcs)
            .include(&self.port_dir)
            .include(&self.mp_dir)
            .include(&self.out_dir)
            .flag("-Os")
            .compile("mpv5");
    }
}

fn rerun_if_changed(manifest_dir: &str) {
    let paths = ["port", "link", "micropython/py"];

    for path in paths.iter().map(|p| format!("{manifest_dir}/{p}")) {
        println!("cargo::rerun-if-changed={path}");
    }
}

fn link_objects(manifest_dir: &str) {
    println!("cargo::rustc-link-search=native={}/link", manifest_dir);
    println!("cargo::rustc-link-arg=-Tvenice.ld");
    // needed for the following symbols as of 2026-01-03: acoshf, asinhf, nearbyintf, atanhf, lgammaf
    println!("cargo::rustc-link-lib=m");
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let builder = Builder::new(manifest_dir);
    builder.gen_version_header();
    builder.gen_headers();
    builder.gen_qstrs_rs();
    builder.compile_mp();

    link_objects(manifest_dir);
    rerun_if_changed(manifest_dir);
}
