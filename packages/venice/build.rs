use std::path::Path;

use headergen::HEADERGEN_DIR_NAME;

fn rerun_if_changed(manifest_path: &Path) {
    let paths = ["port", "link", "micropython/py", HEADERGEN_DIR_NAME];

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

    let build_dir = manifest_path.join(HEADERGEN_DIR_NAME);
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

    let c_srcs = headergen::collect_c_srcs(&mp_dir, &py_dir, &port_dir)
        .expect("couldn't collect c source files");

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
