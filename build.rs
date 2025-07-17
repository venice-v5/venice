use std::process::Command;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    println!("cargo::rustc-link-search=native={manifest_dir}/link");
    println!("cargo::rustc-link-lib=c");
    println!("cargo::rustc-link-lib=m");

    let out_dir = std::env::var("OUT_DIR").unwrap();
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

    println!("cargo::rustc-link-arg={libmpyv5}");
}
