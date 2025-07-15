use std::process::Command;

fn get_file(name: &str, env_var_name: &str) -> String {
    std::env::var(env_var_name).unwrap_or_else(|_| {
        String::from_utf8(
            Command::new("arm-none-eabi-gcc")
                .args([
                    "-mcpu=cortex-a9",
                    "-mfpu=neon-fp16",
                    "-mfloat-abi=hard",
                    "-mthumb",
                    "-nostdlib",
                    "-nostartfiles",
                    &format!("-print-file-name={name}"),
                ])
                .output()
                .unwrap_or_else(|_| panic!("couldn't find {name}, try setting {env_var_name} or making sure `arm-none-eabi-gcc` is installed"))
                .stdout
        )
        .unwrap()
    })
}

#[allow(dead_code)]
fn rerun_if_changed() {
    println!("cargo::rerun-if-env-changed=VENICE_LIBM_PATH");
    println!("cargo::rerun-if-env-changed=VENICE_LIBMPYV5_PATH");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("cargo::rerun-if-changed={manifest_dir}/link/v5.ld");
    println!("cargo::rerun-if-changed={manifest_dir}/port/Makefile");
    println!("cargo::rerun-if-changed={manifest_dir}/port/mpconfigport.h");
    println!("cargo::rerun-if-changed={manifest_dir}/port/mphalport.h");

    let mpy_core =
        std::fs::read_dir(format!("{manifest_dir}/micropython/py")).unwrap_or_else(|err| {
            panic!("couldn't read directory {manifest_dir}/micropython/py: {err}")
        });
    for entry in mpy_core.map(|entry| {
        entry.unwrap_or_else(|err| {
            panic!("couldn't read file in {manifest_dir}/micropython/py: {err}")
        })
    }) {
        let path = entry.path();
        println!(
            "cargo::rerun-if-changed={}",
            path.to_str().expect("invalid UTF-8 path")
        );
    }
}

fn main() {
    // rerun_if_changed();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!("cargo::rustc-link-search=native={manifest_dir}/link");

    let libm = get_file("libm.a", "VENICE_LIBM_PATH");

    println!("cargo::rustc-link-arg={libm}");

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
