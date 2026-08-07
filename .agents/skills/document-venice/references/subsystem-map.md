# Subsystem and counterpart map

Read the vexide revision from `packages/venice/Cargo.toml` and inspect donor files at that revision rather than assuming the local checkout's `HEAD` matches. A newer checkout may contain useful prose improvements, but treat them as secondary material and re-verify all claims against Venice.

## Counterpart map

| Venice source | vexide donor source | Adaptation notes |
| --- | --- | --- |
| `battery.rs` | `vexide-devices/src/battery.rs` | `battery` is a submodule; functions are renamed `get_*`. |
| `color.rs` | `vexide-devices/src/color.rs` | Root Python wrapper with RGB attributes, constants, construction, equality, and `as_int`; no direct Python equivalent of Rust conversions. |
| `motor/mod.rs` and `motor/*.rs` | `vexide-devices/src/smart/motor.rs`, `vexide-devices/src/math.rs` | One Python `Motor` API with default and static constructors; associated classes are split into files; methods/properties and units differ. |
| `rotation_sensor.rs` | `vexide-devices/src/smart/rotation.rs` | Close semantics; use Venice getters, unit arguments, direction constants, and `_MS` constants. |
| `distance_sensor/*.rs` | `vexide-devices/src/smart/distance.rs` | This undocumented subsystem is missing from the user's original checklist. Venice omits vexide's `wait_ready`; result data is flattened into `DistanceObject` attributes. |
| `gps.rs` | `vexide-devices/src/smart/gps.rs` | Close semantic source; Python returns `Point2`, `Vec3`, `Quaternion`, and `EulerZYX` wrappers. |
| `imu.rs` | `vexide-devices/src/smart/imu.rs` | Document calibration awaitability, orientation constants, wrapper exceptions, and unit conversions. |
| `optical/mod.rs`, `optical/*.rs` | `vexide-devices/src/smart/optical.rs` | Associated RGB/raw/gesture records are flattened Python classes with documented attributes. |
| `vision/mod.rs`, `vision/*.rs` | `vexide-devices/src/smart/vision.rs` | Associated Rust sums become multiple Python classes or mode objects; inspect each wrapper and exact exported name. |
| `ai_vision/mod.rs`, `ai_vision/*.rs` | `vexide-devices/src/smart/ai_vision.rs` | Object variants become four Python classes. Verify IDs, flags, color slots, model fields, and reachability before writing examples. |
| `electromagnet.rs` | `vexide-devices/src/smart/electromagnet.rs` | Already documented; a concise device-style reference. |
| `link.rs` | `vexide-devices/src/smart/link.rs` | Device semantics transfer, but `read`, `read1`, `write`, `write1`, `flush`, and `ioctl` are MicroPython stream bindings and need Python/MicroPython semantics. |
| `serial.rs` | `vexide-devices/src/smart/serial.rs` | `SerialPort.open` returns a Venice awaitable. Stream methods come from MicroPython, not vexide's byte helper APIs. |
| `controller/mod.rs`, `controller/id.rs`, `controller/state.rs` | `vexide-devices/src/controller.rs` | State structs become read-only records; screen methods and their awaitable `ControllerFuture` are wrapper-specific. Distinguish methods that return a future from methods that discard it. |
| `adi/*.rs` | matching `vexide-devices/src/adi/*.rs`; expander hardware also uses `smart/expander.rs` | Constructors accept `str | AdiExpanderPort`; wrapper defaults, paired-port validation, runtime TPR, scaled/raw values, and future behavior override Rust signatures. |
| `display.rs` | `vexide-devices/src/display.rs`, plus `color.rs` and `math.rs` | Venice exposes a `display` submodule of functions instead of a `Display` object and shape API. Rebuild docs around actual facade signatures, coordinates, buffers, render modes, fonts, scrolling, and touch snapshots. |
| `math.rs` | `vexide-devices/src/math.rs` and the `mint` type semantics used there | Venice wrappers expose mutable attributes and selected operators. The donor types do not define the complete Python contract. |
| `units/rotation.rs`, `units/time.rs` | vexide `Angle` and Rust `Duration` concepts | Venice-defined singleton classes plus root aliases; no 1:1 API. Use wrapper implementation and established call sites. |
| `competition.rs` | `vexide-core/src/competition.rs` | Venice provides decorators and an awaitable state-machine runtime rather than vexide's generic builder/runtime API. Use Venice implementation first. |
| `vasyncio/*.rs` | concepts in `vexide-async/src/{task,time,executor}.rs` | Bespoke MicroPython cooperative scheduler. Document `EventLoop`, `Task`, `Sleep`, `run`, `spawn`, and `get_running_loop` from Venice behavior. |

## Scope tiers from the project checklist

- High effort: AI Vision, Controller, Display, ADI, `vasyncio`.
- Medium effort: Serial, Competition, Math.
- Low effort: Vision associated classes, Motor associated classes, Color, Units.
- Unlisted but undocumented: Distance Sensor and `DistanceObject`.
- Partially documented despite being marked complete: stream methods in `RadioLink`; associated classes under Optical and Vision; associated classes under Motor; some `free()` methods and protocol-only future types.

Always inventory the current source rather than assuming this list remains current.

## Required pre-documentation gates

Stop and report these as code or reachability issues instead of writing misleading docs:

1. The flat stub output is an intentional maintainer-facing inventory, not a model of runtime namespaces. Verify `battery`, `display`, and `vasyncio` ownership from their dictionaries.
2. The generator can emit annotated classes that are associated/type-only or unreachable; an emitted class is not proof of runtime importability.
3. The checked-in `stubs/venice/__init__.py` is historical and stale relative to current bindings. Generate into `/tmp`; don't overwrite it unless explicitly asked.
4. Repeated flat names such as Vision's `Auto` and `Manual` can represent distinct associated variants. Resolve them from their source file and owning runtime class rather than treating the duplicate inventory names as an API defect.

## Runtime namespace anchors

Before describing import paths, inspect:

- root: `packages/venice/src/modvenice/mod.rs`, the `venice_globals` dictionary;
- battery: `BATTERY_DICT` in `battery.rs`;
- display: `DISPLAY_DICT` in `display.rs`;
- vasyncio: `VASYNCIO_DICT` in `vasyncio/mod.rs`.

Classify each item as one of:

- **root importable** — exported by `venice_globals`;
- **submodule importable** — exported by a submodule dictionary;
- **associated/type-only** — returned, awaited, or accepted by public APIs but not directly exported;
- **protocol-only** — user-visible behavior implemented by macros but not emitted as an ordinary member;
- **internal or unreachable** — not part of a usable Python path.

Document the first four when relevant. Do not advertise the fifth.
