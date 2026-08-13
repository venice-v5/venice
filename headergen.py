#!/usr/bin/env python3

import os
import re
import subprocess
from pathlib import Path


class Builder:
    def __init__(self, venice_dir: Path, additional_crates: list[Path]) -> None:
        self.headergen_dir = venice_dir / "headergen"
        self.headergen_dir.mkdir(parents=True, exist_ok=True)

        self.mp_dir = venice_dir / "micropython"
        self.py_dir = self.mp_dir / "py"
        self.port_dir = venice_dir / "port"
        self.genhdr_dir = self.headergen_dir / "genhdr"
        self.genhdr_dir.mkdir(parents=True, exist_ok=True)

        self.c_srcs = [
            *self.py_dir.glob("*.c"),
            *self.port_dir.glob("*.c"),
            self.mp_dir / "shared/readline/readline.c",
        ]
        self.rust_srcs = list((venice_dir / "src").rglob("*.rs"))
        for crate in additional_crates:
            self.rust_srcs.extend((crate / "src").rglob("*.rs"))

    def gen_version_header(self) -> None:
        subprocess.run(
            ["python3", self.py_dir / "makeversionhdr.py", self.genhdr_dir / "mpversion.h"],
            check=True,
        )

    def gen_qstrdefs(self, qstrs: list[bytes]) -> None:
        qstrdefs_path = self.genhdr_dir / "qstrdefs.preprocessed.h"
        with qstrdefs_path.open("w") as qstrdefs:
            qstrdefs.write("QCFG(BYTES_IN_LEN, (1))\nQCFG(BYTES_IN_HASH, (2))\n")
            for qstr in qstrs:
                qstrdefs.write(f"Q({qstr.decode()})\n\n")

        generated_qstrs = subprocess.run(
            ["python3", self.py_dir / "makeqstrdata.py", qstrdefs_path],
            check=True,
            stdout=subprocess.PIPE,
        ).stdout
        (self.genhdr_dir / "qstrdefs.generated.h").write_bytes(generated_qstrs)

    def gen_moduledefs(self, moduledefs: list[bytes]) -> None:
        collected_path = self.genhdr_dir / "moduledefs.collected"
        with collected_path.open("wb") as collected:
            for moduledef in moduledefs:
                collected.write(moduledef + b"\n")

        moduledefs_h = subprocess.run(
            ["python3", self.py_dir / "makemoduledefs.py", collected_path],
            check=True,
            stdout=subprocess.PIPE,
        ).stdout
        (self.genhdr_dir / "moduledefs.h").write_bytes(moduledefs_h)

    def gen_root_pointers(self, root_pointers: list[bytes]) -> None:
        with (self.genhdr_dir / "root_pointers.h").open("wb") as output:
            for root_pointer in root_pointers:
                output.write(root_pointer + b";\n")

    def gen_headers(self) -> None:
        qstrs: list[bytes] = []
        moduledefs: list[bytes] = []
        root_pointers: list[bytes] = []

        c_qstr_re = re.compile(rb"MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*)")
        c_moduledef_re = re.compile(
            rb"(?:MP_REGISTER_MODULE|MP_REGISTER_EXTENSIBLE_MODULE|MP_REGISTER_MODULE_DELEGATION)\(.*?,\s*.*?\);"
        )
        c_root_pointer_re = re.compile(rb"MP_REGISTER_ROOT_POINTER\((.*?)\);")

        config_headers = [self.py_dir / "mpconfig.h", self.port_dir / "mpconfigport.h"]
        for c_src in [*self.c_srcs, *config_headers]:
            output = subprocess.run(
                [
                    "clang",
                    "-E",
                    "-I",
                    self.port_dir,
                    "-I",
                    self.mp_dir,
                    "-I",
                    self.headergen_dir,
                    "-DNO_QSTR",
                    c_src,
                ],
                check=True,
                stdout=subprocess.PIPE,
            ).stdout
            qstrs.extend(c_qstr_re.findall(output))
            moduledefs.extend(match.group(0) for match in c_moduledef_re.finditer(output))
            root_pointers.extend(c_root_pointer_re.findall(output))

        rust_qstr_re = re.compile(rb"qstr!\(([a-zA-Z_][a-zA-Z0-9_]*)\)")
        method_ident_re = re.compile(
            rb"#\[method.*]\s*(?:#\[stub.*\])?\s*(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"
        )
        constant_ident_re = re.compile(
            rb"#\[constant\]\s*(?:#\[stub.*\])?\s*(?:pub\s+)?const\s+([a-zA-Z_][a-zA-Z0-9_]*)"
        )
        for rust_src in self.rust_srcs:
            source = rust_src.read_bytes()
            for pattern in (rust_qstr_re, method_ident_re, constant_ident_re):
                qstrs.extend(pattern.findall(source))

        self.gen_qstrdefs(qstrs)
        self.gen_moduledefs(moduledefs)
        self.gen_root_pointers(root_pointers)

    def gen_qstrs_rs(self) -> None:
        generated_header = (self.genhdr_dir / "qstrdefs.generated.h").read_text()
        qdef0_re = re.compile(
            r'QDEF0\(MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*), \d+, \d+, ".*"\)'
        )
        qdef1_re = re.compile(
            r'QDEF1\(MP_QSTR_([a-zA-Z_][a-zA-Z0-9_]*), \d+, \d+, ".*"\)'
        )
        defs = [*qdef0_re.findall(generated_header), *qdef1_re.findall(generated_header)]

        output_path = self.headergen_dir / "generated_qstrs.rs"
        with output_path.open("w") as output:
            output.write(
                "#[allow(non_camel_case_types, dead_code)]\n"
                "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]\n"
                "#[repr(C)]\n"
                "pub enum GeneratedQstr {\n"
                "    MP_QSTRnull,\n"
                "    MP_QSTR_,\n"
            )
            for definition in defs:
                output.write(f"    MP_QSTR_{definition},\n")
            output.write("}\n")

        print(output_path)


def main() -> None:
    venice_dir = Path(__file__).resolve().parent / "packages/venice"
    additional_crates = [
        Path(path.strip())
        for path in os.environ.get("HEADERGEN_ADDITIONAL_CRATES", "").split(",")
        if path.strip()
    ]

    builder = Builder(venice_dir, additional_crates)
    builder.gen_version_header()
    builder.gen_headers()
    builder.gen_qstrs_rs()


if __name__ == "__main__":
    main()
