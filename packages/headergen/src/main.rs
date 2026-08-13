use std::{
    env::VarError,
    ffi::OsStr,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    writeln,
};

use headergen::{HEADERGEN_DIR_NAME, collect_c_srcs, collect_dir_files};
use regex::bytes::Regex;

struct Builder {
    headergen_dir: PathBuf,
    mp_dir: PathBuf,
    py_dir: PathBuf,
    port_dir: PathBuf,
    genhdr_dir: PathBuf,
    c_srcs: Vec<PathBuf>,
    rust_srcs: Vec<PathBuf>,
}

impl Builder {
    fn new(venice_dir: &Path, additional_crates: &[PathBuf]) -> Self {
        let headergen_dir = venice_dir.join(HEADERGEN_DIR_NAME);
        std::fs::create_dir_all(&headergen_dir).expect("couldn't create headergen build directory");

        let mp_dir = venice_dir.join("micropython");
        let py_dir = mp_dir.join("py");
        let port_dir = venice_dir.join("port");

        let c_srcs =
            collect_c_srcs(&mp_dir, &py_dir, &port_dir).expect("couldn't collect c source files");

        let mut rust_srcs = Vec::new();
        collect_dir_files(
            &venice_dir.join("src"),
            OsStr::new("rs"),
            true,
            &mut rust_srcs,
        )
        .expect("couldn't collect venice source files");

        for additional_crate in additional_crates {
            collect_dir_files(
                &additional_crate.join("src"),
                OsStr::new("rs"),
                true,
                &mut rust_srcs,
            )
            .expect("couldn't collect additional crate source files");
        }

        let genhdr_dir = headergen_dir.join("genhdr");
        std::fs::create_dir_all(&genhdr_dir).expect("couldn't create genhdr dir");

        Builder {
            headergen_dir,
            py_dir,
            mp_dir,
            port_dir,
            genhdr_dir,
            c_srcs,
            rust_srcs,
        }
    }

    fn gen_version_header(&self) {
        Command::new("python3")
            .arg(self.py_dir.join("makeversionhdr.py"))
            .arg(self.genhdr_dir.join("mpversion.h"))
            .status()
            .expect("couldn't generate mp version header");
    }

    fn gen_qstrdefs(&self, qstrs: &[Vec<u8>]) {
        let qstrdefs_file_path = self.genhdr_dir.join("qstrdefs.preprocessed.h");
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
            writeln!(
                &mut qstrdefs_file,
                "Q({})\n",
                str::from_utf8(qstr).expect("non-utf8 qstr")
            )
            .expect("couldn't write to qstrdefs file");
        }

        let generated_qstrs = Command::new("python3")
            .arg(self.py_dir.join("makeqstrdata.py"))
            .arg(&qstrdefs_file_path)
            .output()
            .expect("coulnd't process qstr data")
            .stdout;

        std::fs::write(
            self.genhdr_dir.join("qstrdefs.generated.h"),
            generated_qstrs,
        )
        .expect("couldn't write out qstr data");
    }

    fn gen_moduledefs(&self, moduledefs: &[Vec<u8>]) {
        let moduledefs_collected_path = self.genhdr_dir.join("moduledefs.collected");
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
            .arg(self.py_dir.join("makemoduledefs.py"))
            .arg(&moduledefs_collected_path)
            .output()
            .expect("couldn't generate moduledefs")
            .stdout;

        std::fs::write(self.genhdr_dir.join("moduledefs.h"), &moduledefs_h)
            .expect("couldn't write out moduledefs");
    }

    fn gen_root_pointers(&self, root_pointers: &[Vec<u8>]) {
        let mut root_pointers_h = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.genhdr_dir.join("root_pointers.h"))
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
            self.mp_dir.join("mpconfig.h"),
            self.port_dir.join("mpconfigport.h"),
        ];
        let c_qstr_src = self.c_srcs.iter().chain(config_headers.iter());

        for c_src in c_qstr_src {
            let out = Command::new("clang")
                .arg("-E")
                .arg("-I")
                .arg(&self.port_dir)
                .arg("-I")
                .arg(&self.mp_dir)
                .arg("-I")
                .arg(&self.headergen_dir)
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
        let method_ident_re = Regex::new(
            r#"#\[method.*]\s*(?:#\[stub.*\])?\s*(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"#,
        )
        .unwrap();
        let constant_ident_re = Regex::new(
            r#"#\[constant\]\s*(?:#\[stub.*\])?\s*(?:pub\s+)?const\s+([a-zA-Z_][a-zA-Z0-9_]*)"#,
        )
        .unwrap();

        for rust_src in self.rust_srcs.iter() {
            let out = std::fs::read(rust_src).expect("couldn't read rust source");

            for cap in rust_qstr_re
                .captures_iter(&out)
                .chain(method_ident_re.captures_iter(&out))
                .chain(constant_ident_re.captures_iter(&out))
            {
                qstrs.push(cap[1].to_vec());
            }
        }

        self.gen_qstrdefs(&qstrs);
        self.gen_moduledefs(&moduledefs);
        self.gen_root_pointers(&root_pointers);
    }

    fn gen_qstrs_rs(&self) {
        let qstrdefs_generated_h =
            std::fs::read_to_string(self.genhdr_dir.join("qstrdefs.generated.h"))
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

        let generated_qstrs_rs_path = self.headergen_dir.join("generated_qstrs.rs");
        let mut generated_qstrs_rs = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&generated_qstrs_rs_path)
            .expect("couldn't open generated_qstrs.rs file");

        writeln!(
            &mut generated_qstrs_rs,
            r"#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum GeneratedQstr {{
    MP_QSTRnull,
    MP_QSTR_,"
        )
        .expect("couldn't write out generated qstrs");

        for def in defs.iter() {
            writeln!(&mut generated_qstrs_rs, "    MP_QSTR_{},", def)
                .expect("couldn't write out generated qstrs");
        }

        writeln!(&mut generated_qstrs_rs, "}}").expect("couldn't write oout generated qstrs");

        println!("{}", generated_qstrs_rs_path.display());
    }
}

fn parse_additional_crates_var(var: Option<String>) -> Vec<PathBuf> {
    let mut additional_crates = Vec::new();

    let Some(var) = var else {
        return additional_crates;
    };

    for path in var.split(',') {
        let trimmed_path = path.trim();
        if trimmed_path.is_empty() {
            continue;
        }
        additional_crates.push(PathBuf::from(trimmed_path));
    }

    additional_crates
}

fn main() {
    let venice_dir = PathBuf::from(
        std::env::var_os("HEADERGEN_VENICE_PATH")
            .expect("expected HEADERGEN_VENICE_PATH to be set to venice crate path"),
    );
    let additional_crates =
        parse_additional_crates_var(match std::env::var("HEADERGEN_ADDITIONAL_CRATES") {
            Ok(v) => Some(v),
            Err(e) => match e {
                VarError::NotPresent => None,
                VarError::NotUnicode(_) => {
                    panic!("HEADERGEN_ADDITIONAL_CRATES contains non unicode characters")
                }
            },
        });

    let builder = Builder::new(&venice_dir, &additional_crates);
    builder.gen_version_header();
    builder.gen_headers();
    builder.gen_qstrs_rs();
}
