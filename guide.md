# Testing the dev versions of Venice

Venice relies mostly on two repositories: `venice` (the runtime itself) and `venice-cli`. There are also a few related tools that are useful to have installed:
* `cargo-v5`
* relevant stuff for the vex v5 rustc target (we still use `armv7a-vex-v5` instead of the more recent `thumbv7a-vex-v5`)

Note that for `cargo v5` you will need a fork that supports the toolchain stuff we use, something like this:
```
git clone git@github.com:fibonacci61/cargo-v5 --branch toolchain-env
cd cargo-v5
cargo install --path .
```

These should already be available if you've used vexide before.

Once you have venice and venice-cli installed, building is fairly easy. To build venice, use cargo v5 build like you'd do in a regular vexide project:
```sh
cd path/to/venice
cargo v5 build
```
If all goes well the final step should look something like `Objcopy .../target/armv7a-vex-v5/debug/venice.bin`, giving you the path of the runtime binary. You may want to set an env var to this path to simplify future steps.

If you encounter a NACK in future steps, add `--release` since the smaller final binary might make it less error-prone.

Then, you can use the CLI to upload the runtime and a program. I recommend using the devshell since it handles venv management etc. which is otherwise quite finnicky.
```sh
cd path/to/venice-cli
nix develop
$ maturin develop
$ venice-cli --help
```
The basic usage of the cli is like this:
```sh
$ cd tests/stress-test/ # or any other valid venice project
$ venice-cli --raw-binary=path/to/venice.bin run
```
`run` builds, uploads, and runs the venice project (and reuploads the runtime if necessary). Note that --raw-binary is only available in dev builds. The terminal is also not the best w/r/t handling ctrl+c so you may want to instead use `upload` instead then `cargo-v5` to run the program + monitor terminal.

The only way to force a reupload of the rt right now is to first remove it from the brain. (We don't do checksum checks or similar for simplicity, this will likely be improved in the future.)
```
$ cargo v5 rm user/venice-v0.1.0.bin
```
