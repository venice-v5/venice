# Venice

This is the repository for Venice. Venice is an independent port of Micropython to the VEX V5. You may know of a first-party Micropython runtime through VEXCode, which is maintained by VEX; **Venice is separate**.

This repository hosts the core of Venice; the runtime. For reference, a Venice program is uploaded to the VEX V5 Brain as two parts: 1) the Venice runtime, which is compiled once for every version of Venice and is a binary which loads the Python bytecode of the program from a particular memory address and runs it, and 2) the actual Python bytecode. The bytecode is currently stored using a custom VPT (Venice Program Table) format to map module identifiers to file bytecode, but we are in the midst of a transition to standardizing to CBOR.

The runtime itself does multiple things. It contains code to bind to Micropython via FFI (note that all of Venice is written in Rust), code to load and run the bytecode, and code to implement the devices API (a public Python API that Venice users access to control peripherals connected to the brain such as motors).

There are a few specific parts:

- [stubs](./stubs): Contains Python stubs for the Venice API, including type annotations and docstrings. We generate docs from stubs using pdoc.
- [micropython-rs](./packages/micropython-rs): FFI bindings to Micropython. Not Venice-specific. Note that Micropython is imported as a submodule.
- [venice](./packages/venice): Code for the actual runtime.
  - [modvenice](./packages/venice/src/modvenice): code for the public-facing Venice Python API (which should match the stubs exactly).
  - [modvasyncio](./packages/venice/src/modvasyncio): code for the `venice.vasyncio` module, which is an async executor for Venice.

For examples on how to implement a public-facing Devices API, refer to the following high-quality examples:

- For a class with various methods: [Motor](./packages/venice/src/modvenice/motor/mod.rs)
- For a data-storing class: [ControllerState](./packages/venice/src/modvenice/controller/state.rs)
- For an enum class: [RotationUnit](./packages/venice/src/modvenice/units/rotation.rs)
- For a full module: [`venice`](./packages/venice/src/modvenice/mod.rs)

Note that we use the `vexide_devices` crate from vexide (which is a port of Rust to the VEX V5) for a high-level abstraction; the vast majority of our Devices API simply mirrors the `vexide_devices` API. There are few Python-specific things that our API has to keep in mind:

- Algebraic data types, particularly enums with variants that store data, are implemented as unions. We don't explicitly declare a function as returning a union (that is done in the stubs) but we rather declare one class for each data type and return any of the possible classes.
- Enums are implemented as regular classes with class variables (although they are constant). Enums are opaque, so the class variables the same type as the enum class.
- `Duration`, `Angle`, etc. aren't available in Micropython. While we could implement alternatives through classes, performance would be significantly worsened as classes are stored on the heap. Instead, we use units. For example, if there is an `Detection` class with `x_inches: f32` and `y_inches: f32` and `angle: Angle`, the x- and y-coordinates are implemented regularly as part of `__getattr__`, **but** `angle` is instead implemented as a method `.angle(RotationUnit)` (it is still stored internally as a Rust `Angle`).

There are other repositories as well:

- [venice-cli](https://github.com/venice-v5/venice-cli): contains code for building and uploading Python projects to the brain, linking in the Venice runtime
- [venice-v5.github.io](https://venice-v5/venice-v5.github.io): contains the code for the Astro-based [website/landing page](https://venice.fibn.cc) and currently nonexistent docs.
- [venice-program-table](https://github.com/venice-v5/venice-program-table): contains code for the Venice Program Table format

## Guidelines for working in this repository

- If you are confused, always ask the user for more information. Always look for high-quality examples before starting; don't assume you "know" anything.
- If you complete a change and the LSP returns diagnostics that you know are false, ignore them; the LSP may be slow to update. If you need to definitively test something (e.g., before informing the user that the task was completed), run `cargo build` to ensure that it builds. Afterwards, run `cargo fmt` and `cargo clippy`; don't address the clippy warnings but inform the user if there are any clippy warnings/errors/etc. 
- There is a lot in this repository! When using ripgrep or similar tools, always filter your results to only return Rust files or similar; otherwise, searches may take several seconds to traverse the Micropython C codebase.
- If you encounter something that you are confused by but later understand it, append the knowledge to a section in [this file](./AGENTS.md).
