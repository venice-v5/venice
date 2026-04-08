---
name: write-stubs
description: Write Python stubs for the Venice API. This is used to keep the stubs up-to-date with the actual runtime implementation.
---

Ensure you have read the [AGENTS.md](../../../AGENTS.md) file for general information about the Venice repository and guidelines for working in it.

Venice spans two primary languages (excluding the Micropython codebase, primarily C, which we link into):
- a Rust crate for the core Venice runtime. This includes Micropython startup, the GC global allocator, VPT parsing, devices API, async runtime, etc.
- a Python package. At CI time, the Rust crate is cross-compiled to the VEX brain's target triple; the resulting binary is objcopy'd into the Python package and bundled as an asset in the final wheel. The Python package also contains the stubs for the public-facing API, which are used for type annotations and documentation generation.

Your goal is to keep stubs for a certain part of Venice's exposed Python API up to date. The user will provide you with the relevant API to port (either a file containing API definitions, or a specific class or method to update stubs for).

You will update stubs in __init__.py, which is linked to by the aforementioned AGENTS.md file. Most of your changes should be additive -- e.g., they should add stubs, not delete other stubs. You may need to replace out-of-date stub implementations, however. Your stubs should be typesafe, using concepts from `typing` as necessary, and all have docstrings. (When using an import from `typing`, you may need to update the import at the top of the file.)

When writing docstrings, use the following rules of thumb:
- The vast majority of our API directly mirrors the `vexide_devices` API. If you find that the implementation of a class or method essentially just forwards to a `vexide_devices` item, read the `vexide_devices` source code (available at `/tmp/vexide/packages/vexide-devices`) and reuse its docstrings. There are some things to be careful about here as well:
  - Some docstrings are very brief, but that may be because the entire file (module) is dedicated to that device, so use the module-level docstring in that case. For example, `vexide-devices/src/smart/motor.rs`'s `Motor` struct has a one-line doc, but the module itself has 57 lines of helpful technical information about the motors, so you would want to use the module docstrings. 
  - Compared to Cargo's specific Markdown-inspired docstring syntax, Python's docstrings are pure Markdown, so keep that in mind when porting docstrings. You'll also want to replace references to vexide in docstrings with Venice.
  - Docstrings may reference a specific type using Cargo docstrings' syntax, like [`MyEnum::SomeOption`]. The equivalent in Python is simply `MyEnum.SomeOption` -- the docstrings generator automatically picks those up, without the square brackets and deeplinks them. It's fine if `MyEnum` doesn't exist in the stubs yet; keep track of edge cases like this and compile a report of remaining things to check once you are done with the stubs.
- Otherwise, use your best judgement: write professional, concise docstrings that give needed context without being verbose.

If stubbing a particular class or method requires referencing another type that isn't yet stubbed yet, stop immediately and inform the user.

Stubs are also slightly difficult to implement: the following are some particular cases you should be careful about:

**Indicating that a method is inaccessible:**
In most cases this is implicit but just not stubbing the method, but for special cases like initializers, this is necessary:
```py
from typing import Never

class MyClass:
    # make a regular method inaccessible
    def my_inaccessible_method(self) -> Never:
        ...
    
    # make the initializer inaccessible
    def __new__(cls) -> Never:
        ...
```

**Enums:**
This is a particularly hacky workaround needed to satisfy the docs generator:
```py
from typing import ClassVar

class MyEnum:
    A: ClassVar['MyEnum']
    B: ClassVar['MyEnum']

MyEnum.A = MyEnum()
MyEnum.B = MyEnum()
```
