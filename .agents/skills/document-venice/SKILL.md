---
name: document-venice
description: Documents the Python-facing Venice VEX V5 runtime by adapting Rust documentation from the configured vexide dependency into accurate Rust doc comments and generated Python docstrings. Use when finding undocumented APIs under packages/venice/src/modvenice, documenting a Venice subsystem or associated classes, translating vexide examples to Python, or auditing Venice documentation coverage and links.
compatibility: Requires Python 3, Git, a Venice checkout, and preferably a local vexide checkout at $VEXIDE_REPO, ../vexide, or ~/src/vrc/vexide.
---

# Document Venice APIs

Work one coherent subsystem at a time. Add documentation to the Rust bindings only; the generated Python stub is a validation artifact, not the source of truth.

Read [the style and adaptation guide](references/style-and-adaptation.md) before editing. Read [the subsystem map](references/subsystem-map.md) when choosing donor files, handling a non-1:1 API, or deciding whether a generated type is actually public.

## Hard constraints

- Edit `packages/venice/src/modvenice/**/*.rs`, adding or correcting `///` comments adjacent to exported binding items. Don't hand-edit generated stubs.
- Preserve runtime behavior, public signatures, macro attributes, item order, and formatting unless the user separately authorizes code or generator fixes.
- Treat the Venice wrapper as authoritative and vexide as donor material. Never advertise an upstream API that Venice doesn't expose.
- Use Python-facing names, exceptions, types, units, properties, and examples throughout. Remove every Rust-specific link or code fragment.
- Document associated records, enum-like classes, awaitables, constants, and submodule functions, not only the headline device class.
- If documentation would require pretending that an unreachable type or broken API works, stop that item and report the implementation mismatch. Continue independent items when possible.
- Don't broaden a documentation task into cleanup of completed subsystems. Fix an existing defect only when it is in a touched doc block or prevents a valid cross-link/example.

## 1. Establish the actual Python surface

Read the complete target Venice file and all sibling files in its subsystem. Inspect the runtime dictionaries in `modvenice/mod.rs` and, for submodules, the relevant `*_DICT`. Then inventory:

- classes from `#[class(qstr!(...))]`;
- constructors from `#[make_new]`;
- methods from `#[method]`, including static bindings;
- functions from `#[fun]`;
- constants from `#[constant]`;
- attributes from `#[stub(attrs = [...])]` and `#[attr]`;
- awaitable, iterable, operator, printer, and stream behavior from protocol macros;
- exact signatures from `#[stub(sig = ...)]`;
- defaults and validation from argument parsing;
- exceptions from explicit raises, `?`, and `From<...> for Exception` conversions.

Classify every item as root importable, submodule importable, associated/type-only, protocol-only, internal, or unreachable. The stub generator intentionally emits a flat inventory for maintainers to process by hand and may include annotated but unreachable classes, so generated output alone cannot make this decision.

Generate a temporary stub to expose missing docstrings and exact rendered signatures:

```bash
python3 packages/venice/stubgen.py -o /tmp/venice-generated.pyi
python3 - <<'PY'
import ast
from pathlib import Path
ast.parse(Path('/tmp/venice-generated.pyi').read_text())
print('generated stub parses')
PY
```

Don't overwrite `stubs/venice/__init__.py`; it is currently stale and is updated only by an explicit stub task.

## 2. Read the configured donor documentation

Read the vexide revision from `packages/venice/Cargo.toml`, then resolve the checkout and read the donor file at that revision:

```bash
VEXIDE_REV=$(python3 - <<'PY'
import tomllib
from pathlib import Path

cargo = tomllib.loads(Path("packages/venice/Cargo.toml").read_text())
print(cargo["dependencies"]["vexide-devices"]["rev"])
PY
)

if [ -n "${VEXIDE_REPO:-}" ]; then
  VEXIDE=$VEXIDE_REPO
elif [ -d ../vexide/.git ]; then
  VEXIDE=../vexide
else
  VEXIDE=$HOME/src/vrc/vexide
fi

git -C "$VEXIDE" cat-file -e "$VEXIDE_REV^{commit}"
git -C "$VEXIDE" show "$VEXIDE_REV:packages/vexide-devices/src/smart/ai_vision.rs"
```

Use the map in `references/subsystem-map.md` for the actual donor path. Read the relevant donor type and method docs, not only a same-named file. For Competition use `vexide-core`; for `vasyncio`, Units, Display, and Math, combine donor concepts with the Venice implementation because there is no 1:1 API.

For each Venice item, make a small mental adaptation record:

- donor item and transferable semantics;
- actual Python name/signature/defaults;
- return value and units;
- Python exceptions and validation;
- sync, awaitable, property, or protocol behavior;
- links that need rebinding;
- example that demonstrates the wrapper rather than the Rust API.

Omit donor sections about Rust ownership, traits, generics, lifetimes, `Result`, `Option`, panics, or APIs absent from Venice unless they translate into a real Python-visible constraint.

## 3. Write source doc comments

Attach `///` comments directly to each binding item. A blank line between pending comments and an item can prevent the scanner from associating them.

Cover every public item in the target:

- **Class:** purpose, construction/importability, hardware or semantic overview, attributes, and protocol behavior.
- **Constructor/method/function:** summary, exact parameters in prose, defaults or valid ranges, units, return semantics, side effects, exceptions, and a useful example when behavior isn't obvious.
- **Constant:** meaning, unit, range, and any important relationship to hardware timing or another API.
- **Associated record:** every `#[stub(attrs = ...)]` attribute by exact name, including units and mutability. Attribute docs must currently live in the class docstring because the generator has no individual attribute-doc channel.
- **Awaitable:** what operation it represents, what `await` returns, and whether users construct it directly or receive it from another API.
- **Stream/protocol type:** user-visible behavior and constraints, using Python/MicroPython semantics rather than Rust trait terminology.

Prefer concise docs for obvious values and rich class-level prose for hardware concepts. Don't copy long upstream examples when a small Python example proves the same contract.

## 4. Translate and verify examples

Rebuild each example against the generated Python signature. In particular:

- use `from venice import *` and integer port numbers;
- use `Motor.new_exp(1)` when documenting an EXP motor, not the V5 default constructor;
- use exact enum-like constants such as `Direction.REVERSE`;
- use `MILLIS` and singular `SECOND`;
- await returned future objects;
- call `vasyncio.run(main())` with a coroutine object;
- use `try`/`except DeviceError` or `ValueError` only when error handling matters;
- sleep in indefinite polling loops;
- check every f-string, variable name, branch operator, and copied class name.

A Python block being syntactically valid is necessary but insufficient. Walk through it against the wrapper implementation to confirm that constructors, properties, method names, await behavior, and outputs are semantically possible.

## 5. Run scoped audits

Run the bundled audit from the repository root with a regular expression matching the completed subsystem:

```bash
python3 .agents/skills/document-venice/scripts/audit_stub_docs.py \
  --match '^(AiVision|AprilTagFamily)' \
  --source packages/venice/src/modvenice/ai_vision
```

Useful match expressions:

| Scope | `--match` expression |
| --- | --- |
| ADI | `^(Adi|PotentiometerType)` |
| AI Vision | `^(AiVision|AprilTagFamily)` |
| Controller | `^(Controller|ButtonState|JoystickState)` |
| Display | `^(RenderMode|FontFamily|FontSize|TouchEvent|draw_|scroll|set_render_mode|render$|erase$|print$|get_touch_status|is_)` |
| Serial | `^SerialPort` |
| Competition | `^Competition` |
| Math | `^(Vec3|Quaternion|EulerZYX|Point2)` |
| Color | `^Color` |
| Units | `^(RotationUnit|TimeUnit)` |
| vasyncio | `^(EventLoop|Sleep|Task|run$|spawn$|get_running_loop$)` |
| Vision associated types | `^(VisionCode|LedMode|VisionMode|VisionObject|VisionSignature|DetectionSource|WhiteBalance|Auto|Manual|StartupAuto|Signature|Code|Line)` |
| Motor associated types | `^(BrakeMode|Direction|Gearset|MotorType)` |
| Distance | `^Distance` |

The audit checks generated docstring presence, parameter/attribute name coverage, dotted Python links, common Rust residue, exception headings, and likely missing f-string prefixes. Each `--source` file or directory is checked directly for Python-fence syntax because the current stub generator strips code indentation; repeat `--source` when a subsystem spans separate paths. The audit is intentionally heuristic, so inspect any finding against the source rather than weakening valid documentation to satisfy the checker.

Then run:

```bash
git diff --check
cargo fmt --all -- --check
```

Run `cargo check -p venice` when the environment has the required V5 toolchain or when anything besides comments changed. If the cross toolchain is unavailable, say so; don't claim a build passed.

## 6. Review before declaring a subsystem complete

Read every changed doc block and the corresponding generated stub docstring. Confirm:

- every target class, constructor, method, function, constant, associated record, and awaitable is covered;
- every non-`self` parameter is described by exact Python name;
- attribute names, units, mutability, ranges, and coordinate conventions are explicit;
- all dotted links resolve to real Python names with exact casing;
- no Rust path, syntax, type, panic, or error variant remains;
- each `# Raises` entry matches the wrapper's real exception path;
- examples are valid Python and semantically match the wrapper;
- imports and direct construction claims match runtime dictionaries;
- unrelated source behavior and signatures are unchanged.

Report changed files, APIs covered, validation commands and results, implementation/generator defects discovered, and any items deliberately left unresolved. A subsystem isn't complete while its scoped audit has unexplained failures.
