use std::path::{Path, PathBuf};

fn collect_c_srcs(mp_dir: &Path, py_dir: &Path, port_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut c_srcs = Vec::new();

    for dir in [py_dir, port_dir] {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|extension| extension == "c") {
                c_srcs.push(path);
            }
        }
    }

    c_srcs.push(mp_dir.join("shared/readline/readline.c"));
    Ok(c_srcs)
}

fn rerun_if_changed(manifest_path: &Path) {
    let paths = ["port", "link", "micropython/py", "headergen"];

    for path in paths.iter().map(|p| manifest_path.join(p)) {
        println!("cargo::rerun-if-changed={}", path.display());
    }
}

fn link_objects(manifest_dir: &str) {
    println!(
        "cargo::rustc-link-search=native={}",
        Path::new(manifest_dir).join("link").display()
    );
    println!("cargo::rustc-link-arg=-Tvenice.ld");
    // needed for the following symbols as of 2026-01-03: acoshf, asinhf, nearbyintf, atanhf, lgammaf
    println!("cargo::rustc-link-lib=m");
    println!("cargo::rustc-link-lib=c");
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = Path::new(manifest_dir);

    let build_dir = manifest_path.join("headergen");
    let generated_qstrs_rs = build_dir.join("generated_qstrs.rs");

    if !std::fs::exists(&generated_qstrs_rs)
        .expect("can't check existence of generated_qstrs.rs in headergen")
    {
        panic!("generated_qstrs.rs not found; run headergen before compiling venice");
    }

    println!(
        "cargo::rustc-env=GENERATED_QSTRS_RS={}",
        generated_qstrs_rs.display()
    );

    let mp_dir = manifest_path.join("micropython");
    let py_dir = mp_dir.join("py");
    let port_dir = manifest_path.join("port");

    let c_srcs =
        collect_c_srcs(&mp_dir, &py_dir, &port_dir).expect("couldn't collect c source files");

    let mut build = cc::Build::new();

    build
        .files(&c_srcs)
        .include(&port_dir)
        .include(&mp_dir)
        .include(&build_dir)
        .flag("-Os")
        .compile("mpv5");

    link_objects(manifest_dir);
    rerun_if_changed(manifest_path);
}
