# Venice documentation style and adaptation guide

## Source-of-truth order

Use evidence in this order whenever sources disagree:

1. Runtime dictionaries define where a Python name is importable: root exports are in `packages/venice/src/modvenice/mod.rs`; submodule dictionaries are in `battery.rs`, `display.rs`, and `vasyncio/mod.rs`.
2. Binding macros define Python-visible items: `#[class]`, `#[make_new]`, `#[method]`, `#[fun]`, `#[constant]`, `#[attr]`, and protocol macros such as `#[iter]` and `#[stream]`.
3. `#[stub(...)]` defines the intended Python signature when Rust inference is insufficient.
4. The Venice wrapper implementation defines defaults, conversions, ranges, mutability, side effects, exceptions, and awaitability.
5. The vexide source at the revision configured in `packages/venice/Cargo.toml` supplies donor prose and hardware semantics.
6. Existing completed Venice docs supply style, but they are examples rather than authority and contain known defects.

Never change runtime behavior, macro attributes, public signatures, or build-sensitive ordering merely to make copied documentation fit. Report an implementation or generator mismatch instead.

## Standard shape

Start with a direct present-tense summary. Use “Returns…”, “Sets…”, “Creates…”, or a noun phrase for a class or constant. Add only details needed for correct use: units, ranges, coordinate conventions, persistence, timing, side effects, prerequisites, and hardware limitations.

Class docs may use descriptive sections such as `# Hardware Overview`, `# Coordinate System`, `# Accuracy`, or `# Volatile Memory`. Method docs normally follow this shape:

````rust
/// Sets the sensor's data interval.
///
/// `interval` is measured in the supplied `unit` and must be at least ...
///
/// # Raises
///
/// - `ValueError`: If `interval` is negative.
/// - `DeviceError`: If no compatible device is connected.
///
/// # Examples
///
/// ```python
/// from venice import *
///
/// sensor = ExampleSensor(1)
/// sensor.set_data_interval(10, MILLIS)
/// ```
````

Document arguments in natural prose using their exact backticked Python names. Add a short `# Arguments` list only when prose would become awkward; completed Venice docs generally don't use boilerplate argument or return sections. The opening sentence documents the return value.

Standardize exception headings on `# Raises`. Name Python exceptions and the exact condition. Collapse Rust `Result<T, E>` into a normal Python return plus raised exceptions. Smart Port disconnection/type mismatches commonly become `DeviceError`; argument validation and use-after-free commonly become `ValueError`, but verify the wrapper's conversion path every time.

Use Markdown links for external sources and inline code for Python symbols. Dotted references must use exact Python casing, such as `Motor.set_current_limit`, `Direction.FORWARD`, or `RotationSensor.MIN_DATA_INTERVAL_MS`. Never leave `Self`, `::`, Rust enum casing, or links to Rust-only traits and error variants.

## Python example rules

Every user example must:

- use a `python` fence and valid Python syntax;
- normally start with `from venice import *`;
- use integer Smart Port numbers rather than `peripherals.port_1`;
- use Python names, properties, `True`/`False`/`None`, `and`/`or`, and `print(...)`;
- call the public wrapper API exactly as its generated signature exposes it;
- include `await` for a Venice awaitable and pass a coroutine object to `vasyncio.run`, normally `vasyncio.run(main())`;
- yield in polling loops with `await vasyncio.Sleep(value, MILLIS)` or `SECOND`;
- avoid hardware claims or outputs that cannot be inferred from the implementation.

Typical Rust-to-Python rewrites are semantic, not textual:

| vexide/Rust | Venice/Python |
| --- | --- |
| `use vexide::prelude::*` | `from venice import *` |
| `peripherals.port_1` / `SmartPort` | `1` / `int` |
| `Type::new(...)` | `Type(...)` or the wrapper's static constructor |
| `println!` | `print` |
| `if let Ok(value)` | direct call, optionally inside `try`/`except` |
| `Option<T>` | `T | None` |
| `Result<T, E>` | `T`, with `# Raises` |
| `Duration::from_millis(10)` | `10, MILLIS` |
| `Angle` | numeric value plus a `RotationUnit` when the wrapper requires one |
| `sleep(...).await` | `await vasyncio.Sleep(...)` |
| `obj.method()` | `obj.method()` or `obj.property`; inspect the wrapper |

## Units and values

State fixed physical units in the summary or semantic paragraph: volts, amps, watts, newton-metres, RPM, degrees Celsius, metres, or degrees per second. State normalized ranges precisely, including interval bounds such as `[0.0, 1.0]` and `[0.0, 360.0)`.

Venice represents convertible values as a numeric argument plus `RotationUnit` or `TimeUnit`. Examples normally use the root aliases `DEGREES`, `RADIANS`, `TURNS`, `MILLIS`, and singular `SECOND`. Fixed-unit constants carry a suffix such as `_MS` where the API defines one; never invent a suffix.

## Attributes, associated types, and protocols

`#[stub(attrs = [...])]` currently has no per-attribute doc channel. Document every listed attribute by exact name in the owning class docstring, including type/units, mutability, and meaning. Do not change the generator as part of a documentation-only task.

Document associated records, enum-like classes, and awaitable future classes when users receive them, even if they aren't directly constructible. Say that construction is indirect when relevant. Don't claim a type is importable until it appears in a runtime dictionary.

The stub generator currently omits some protocol behavior. Explain user-visible awaitability, arithmetic, equality, iteration, and stream behavior in class prose when the corresponding macro or implementation exists, but don't invent explicit dunder methods in documentation or stubs.

## Known defects in completed examples

Completed files are useful but not clean templates. Do not copy these defects:

- `imu.rs` has a missing `await`, a Rust `println!`, `# Exceptions`, and the invalid link `InertialSensor.Calibrate`.
- `optical/mod.rs` has Rust `&&` in a Python block and unrelated copied `free()` names.
- `vision/mod.rs` constructs `VisionSensor` where `VisionSignature` is intended in one example.
- `motor/mod.rs` has an incorrect EXP constructor example and a copied “date write interval” typo.
- `battery.rs` has a non-f-string containing `{voltage}`.
- `rotation_sensor.rs` contains incorrect casing and stale constant links.
- `link.rs` and `serial.rs` leave MicroPython stream methods undocumented.

Use `battery.rs` as the brevity baseline, `electromagnet.rs` for a concise device, and `gps.rs`, `imu.rs`, and `motor/mod.rs` for richer structure, then verify every borrowed detail against the current wrapper.
