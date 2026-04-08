# Venice

<!--[![GitHub Release](https://img.shields.io/github/v/release/venice-v5/venice)](https://github.com/venice-v5/venice)-->
<!-- the badge below is a placeholder until v0.1.0 is released and we can use the badge above -->
![GitHub Release](https://img.shields.io/badge/release-v0.1.0--alpha-yellow)
[![API Reference](https://img.shields.io/badge/API-Reference-007acc?style=flat-square&logo=visual-studio-code&logoColor=white)](https://your-subdomain.yourwebsite.com/api)
![GitHub License](https://img.shields.io/github/license/venice-v5/venice)
[![Discord](https://img.shields.io/discord/1385488860661678171.svg?label=discord&logo=discord&color=7289DA&logoColor=white)](https://discord.gg/UhGmfY28)

<!-- TODO: add more details -->
**Venice** is a modern alternative to VEXcode Python.

## Features

- **Multi-File Projects & External Libraries**: Venice lets you organize your code into multiple files and cleanly import them. Want to use an external library for odometry or PID? Just drop the files into your project and import them.
- **Intuitive Robot Control**: `import venice` gives you a robust, Pythonic library for controlling your VEX V5 devices, and exposes functions and devices not present in VEXcode.
- **Efficient Multitasking**: Venice uses modern `async`/`await` multitasking, with the efficient custom-made `vasyncio` event loop, letting you run dozens of tasks without freezing your robot.
- **Predictable, Immediate-Mode Control**: Instead of registering callbacks for events like button presses that are called in the background unpredictably, Venice hands you the steering wheel. You read controller inputs exactly when you want to in your main loops.
- **Optimized Math for Autonomous**: Venice internally optimizes the representation of numbers in memory, letting your math become even faster and your program consumes less RAM.
- **Industry-Standard Tooling**: Use [`venice-cli`](https://github.com/venice-v5/venice-cli/) for project scaffolding, building, and uploading. It integrates seamlessly with standard Python project structures and modern dev environments.
- **Smart Auto-Complete & Type-Hinting**: The `venice` PyPI package provides complete, type-safe, and documented stubs for all Venice APIs.

## Getting Started

TBA

## Competition Template

```python
# Import all Venice devices and submodules.
from venice import *

# Initialize your devices here.
motor = Motor(1)
imu = InertialSensor(2)

# Create a competition template.
comp = Competition()

# Add your driver routine.
@comp.driver
async def driver():
    print("Driver control!")
    while True:
        motor.set_velocity(200)
        
        # IMPORTANT: Make sure you sleep in your loops! 5-10 milliseconds is the recommended duration.
        await vasyncio.Sleep(10, MILLIS)
    
# Add your autonomous routine.
@comp.autonomous
async def auton():
    print("Autonomous!")
    await vasyncio.Sleep(1000, MILLIS)

# Define your `async` entrypoint. This is where you should put your initialization logic. (e.g. calibration)
async def main():
    await imu.calibrate()
    
    # Start the competition runtime. Now your routines will be run until the end of the program.
    await comp.run()

# Finally, spin up an `async` runtime and start executing your `main` function.
vasyncio.run(main())
```

## Documentation

An API reference for the `venice` module is available at https://venice.fibn.cc/reference/.

## Development

### Project Structure

Venice is currently composed of four Rust packages under the `./packages` directory:
- `venice`: Runtime binary and Python `venice` module
- `micropython-rs`: High-level, hand-written MicroPython bindings. These bindings are only compatible with the port used by Venice.
- `argparse`: Python argument parsing framework with clean error message handling
- `micropython-macros`: Proc-macros for generating MicroPython classes with clean Rust syntax

Venice uses the [`vexide-devices`](https://github.com/vexide/vexide/) and [`vex-sdk`](https://github.com/vexide/vex-sdk/) crates to control VEX devices.

### Building

Building the Venice runtime requires a specialized cross-compilation environment:

1. **Toolchain**: Venice is built using the LLVM ARM toolchain, whose installation can be automated with a special [fork](https://github.com/fibonacci61/cargo-v5/tree/toolchain-env) of the `cargo-v5` tool on the `toolchain-env` branch. (The `main` branch will not work, you must use the `toolchain-env` branch on @fiboancci61's fork.)
2. **direnv**: Use [`direnv`](https://direnv.net/) to execute the `direnv` script in `./packages/venice/.envrc`. The script will call into `cargo-v5` and put the ARM toolchain into scope.
3. **Python 3**: The build script relies on [Python 3](https://www.python.org/downloads/) to execute other scripts for code generation (QSTRs, module definitions, and root pointers).

Before you start building, run `cargo v5 toolchain install` in `./packages/venice` to install the ARM toolchain. Then, run `direnv allow` in the project root to let the `direnv` put the toolchain into scope as long as you are in the project directory.

Finally, to build the Venice runtime, run `cargo v5 build` in `./packages/venice`, and the runtime binary will be generated in your target directory (`./target/armv7a-vex-v5/release/venice.bin`).
