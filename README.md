# Venice

<!--[![GitHub Release](https://img.shields.io/github/v/release/venice-v5/venice)](https://github.com/venice-v5/venice)-->
<!-- the badge below is a placeholder until v0.1.0 is released and we can use the badge above -->
![GitHub Release](https://img.shields.io/badge/release-v0.1.0--alpha-yellow)
[![API Reference](https://img.shields.io/badge/API-Reference-007acc?style=flat-square&logo=visual-studio-code&logoColor=white)](https://venice.fibn.cc/reference/)
![GitHub License](https://img.shields.io/github/license/venice-v5/venice)
[![Discord](https://img.shields.io/discord/1385488860661678171.svg?label=discord&logo=discord&color=7289DA&logoColor=white)](https://discord.gg/UhGmfY28)

<!-- TODO: add more details -->
**Venice** is a modern Micropython runtime for the VEX V5 brain and platform as an alternative to VEXcode Python.

## Features
* To add Venice to a project, you can just install it! The [Venice CLI](https://github.com/venice-v5/venice-cli) and runtime SDK are available as a regular PyPI package. This is in contrast to VEX Python, which is only available through VEXcode or a proprietary VSCode extension.
* All of your program's metadata is stored in the industry-standard pyproject.toml config file, compared to VEX Python's custom configuration formula.
* Multi-file support is built-in to Venice, just like any other Python project. In VEX Python, you can only have one file, which is a dealbreaker for many teams.
* Venice takes advantage of modern Python features, such as advanced typing annotations, `async`/`await` multitasking, idiomatic APIs, and more. The VEX Python SDK is unidiomatic, preventing integration with the broader Python ecosystem.
* Venice can be used everywhere! You can write Venice code in a code editor of your choice; the Venice CLI is just a regular executable package that you can run from anywhere with a terminal.
* Venice is designed for speed, with bytecode compiled at build-time to decrease startup latency and MicroPython math optimizations enabled to make math-heavy calculations faster.

## Getting Started

First, install `uv` if you haven't already using [these instructions](https://docs.astral.sh/uv/getting-started/installation/).

```
uvx venice-cli new my-project
```

This creates a Venice project with `venice` and `venice-cli` installed, with a minimal `main.py`:
```py
from venice import *
import vasyncio

async def main():
    print("Hello, Venice!")

vasyncio.run(main())
```

To upload programs, connect your computer to a controller or brain. From the `my-project` directory, run
```
uv run venice-cli run
```
which uploads and runs your code, then opens the terminal to view output.

To see other commands, run
```
uv run venice-cli help
```

Note that you can't just run `venice-cli`. If you're inside a Venice project, use `uv run venice-cli`. Otherwise, use `uvx venice-cli` (this will only support the `new`, `help`, and `terminal` subcommands). 

## Competition Template

```python
from venice import *

# Initialize your devices here
motor = Motor(1)
imu = InertialSensor(2)

# Create a competition template
comp = Competition()

# Driver routine
@comp.driver
async def driver():
    print("Driver control!")
    while True:
        motor.set_velocity(200)
        
        # IMPORTANT: Make sure you sleep in your loops! 5-10 milliseconds is the recommended duration.
        await vasyncio.Sleep(10, MILLIS)
    
# Autonomous routine
@comp.autonomous
async def auton():
    print("Autonomous!")
    await vasyncio.Sleep(1000, MILLIS)

# Define your `async` entrypoint. This is where you should put your initialization logic (e.g. calibration)
async def main():
    await imu.calibrate()
    
    # Start the competition runtime. Now your routines will be run until the end of the program
    await comp.run()

# Create an `async` runtime and start executing your `main` function
vasyncio.run(main())
```

## Documentation

An API reference for the `venice` module is available at https://venice.fibn.cc/reference/. A work-in-progress tutorial series is also available at https://venice.fibn.cc/docs/.

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
