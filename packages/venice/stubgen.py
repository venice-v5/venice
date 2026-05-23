#!/usr/bin/env python3
"""Generate draft Python stubs for the Venice module from Rust API macros."""

from __future__ import annotations

import argparse
import ast
import keyword
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parent
DEFAULT_SRC = ROOT / "src" / "modvenice"


@dataclass
class Attr:
    name: str
    args: str
    raw: str


@dataclass
class StubMeta:
    sigs: list[str] = field(default_factory=list)
    typ: str | None = None
    attrs: list[str] = field(default_factory=list)
    skip: bool = False


@dataclass
class RustParam:
    name: str
    typ: str | None
    is_self: bool = False


@dataclass
class RustFunction:
    name: str
    params: list[RustParam]
    ret: str | None


@dataclass
class FunctionDef:
    name: str
    sigs: list[str]
    docs: list[str] = field(default_factory=list)
    decorators: list[str] = field(default_factory=list)


@dataclass
class ConstantDef:
    name: str
    typ: str
    docs: list[str] = field(default_factory=list)
    source: str = ""


@dataclass
class ImplDef:
    rust_type: str
    source_id: str
    methods: list[FunctionDef] = field(default_factory=list)
    constants: list[ConstantDef] = field(default_factory=list)
    attrs: list[str] = field(default_factory=list)


@dataclass
class ClassDef:
    rust_name: str
    name: str
    source_id: str
    docs: list[str] = field(default_factory=list)
    attrs: list[str] = field(default_factory=list)
    methods: list[FunctionDef] = field(default_factory=list)
    constants: list[ConstantDef] = field(default_factory=list)


@dataclass
class ScanResult:
    classes: list[ClassDef] = field(default_factory=list)
    impls: list[ImplDef] = field(default_factory=list)
    functions: list[FunctionDef] = field(default_factory=list)


def code_part(line: str) -> str:
    """Return line text with strings and comments removed for delimiter scans."""
    out: list[str] = []
    i = 0
    in_string = False
    in_char = False
    escaped = False

    while i < len(line):
        ch = line[i]
        nxt = line[i + 1] if i + 1 < len(line) else ""

        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            out.append(" ")
            i += 1
            continue

        if in_char:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == "'":
                in_char = False
            out.append(" ")
            i += 1
            continue

        if ch == "/" and nxt == "/":
            break
        if ch == '"':
            in_string = True
            out.append(" ")
            i += 1
            continue
        if ch == "'":
            # Treat lifetime names as normal code, but hide character literals.
            if i + 2 < len(line) and line[i + 2] == "'":
                in_char = True
                out.append(" ")
                i += 1
                continue
        out.append(ch)
        i += 1

    return "".join(out)


def delimiter_delta(line: str, open_ch: str, close_ch: str) -> int:
    code = code_part(line)
    return code.count(open_ch) - code.count(close_ch)


def split_top_level(text: str, delimiter: str = ",") -> list[str]:
    parts: list[str] = []
    start = 0
    depth = 0
    in_string = False
    in_char = False
    escaped = False
    pairs = {"(": ")", "[": "]", "{": "}", "<": ">"}
    closing = set(pairs.values())

    for i, ch in enumerate(text):
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            continue
        if in_char:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == "'":
                in_char = False
            continue

        if ch == '"':
            in_string = True
        elif ch == "'" and not (
            i + 1 < len(text) and (text[i + 1].isalpha() or text[i + 1] == "_")
        ):
            in_char = True
        elif ch in pairs:
            depth += 1
        elif ch in closing and depth > 0:
            depth -= 1
        elif ch == delimiter and depth == 0:
            parts.append(text[start:i].strip())
            start = i + 1

    tail = text[start:].strip()
    if tail:
        parts.append(tail)
    return parts


def find_matching(
    text: str, start: int, open_ch: str = "(", close_ch: str = ")"
) -> int:
    depth = 0
    in_string = False
    in_char = False
    escaped = False

    for i in range(start, len(text)):
        ch = text[i]
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            continue
        if in_char:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == "'":
                in_char = False
            continue

        if ch == '"':
            in_string = True
        elif ch == "'" and not (
            i + 1 < len(text) and (text[i + 1].isalpha() or text[i + 1] == "_")
        ):
            in_char = True
        elif ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return i
    return -1


def first_code_delimiter(text: str, delimiter: str) -> int:
    code = code_part(text)
    idx = code.find(delimiter)
    return idx


def read_attribute(lines: list[str], start: int) -> tuple[str, int]:
    raw_lines = [lines[start].strip()]
    balance = delimiter_delta(lines[start], "[", "]")
    i = start
    while balance > 0 and i + 1 < len(lines):
        i += 1
        raw_lines.append(lines[i].strip())
        balance += delimiter_delta(lines[i], "[", "]")
    return "\n".join(raw_lines), i


def parse_attr(raw: str) -> Attr:
    inner = raw.strip()
    if inner.startswith("#["):
        inner = inner[2:]
    if inner.endswith("]"):
        inner = inner[:-1]
    inner = inner.strip()

    paren = inner.find("(")
    if paren == -1:
        return Attr(inner, "", raw)

    end = find_matching(inner, paren)
    if end == -1:
        return Attr(inner[:paren].strip(), inner[paren + 1 :].strip(), raw)
    return Attr(inner[:paren].strip(), inner[paren + 1 : end].strip(), raw)


def attrs_named(attrs: Iterable[Attr], name: str) -> list[Attr]:
    return [attr for attr in attrs if attr.name == name]


def attr_named(attrs: Iterable[Attr], name: str) -> Attr | None:
    for attr in attrs:
        if attr.name == name:
            return attr
    return None


def parse_literal(value: str):
    return ast.literal_eval(value.strip())


def parse_stub_meta(attrs: Iterable[Attr]) -> StubMeta:
    meta = StubMeta()
    for attr in attrs_named(attrs, "stub"):
        if not attr.args:
            continue
        for part in split_top_level(attr.args):
            if part == "skip":
                meta.skip = True
                continue
            if "=" not in part:
                continue
            key, value = part.split("=", 1)
            key = key.strip()
            value = value.strip()
            try:
                parsed = parse_literal(value)
            except (SyntaxError, ValueError):
                continue
            if key == "sig" and isinstance(parsed, str):
                meta.sigs.append(parsed)
            elif key == "type" and isinstance(parsed, str):
                meta.typ = parsed
            elif key == "attrs" and isinstance(parsed, list):
                meta.attrs.extend(str(item) for item in parsed)
    return meta


def qstr_name(args: str) -> str | None:
    match = re.search(r"qstr!\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)", args)
    if match:
        return match.group(1)
    return None


def binding_name(args: str) -> str:
    match = re.search(r'binding\s*=\s*"([A-Za-z_][A-Za-z0-9_]*)"', args)
    return match.group(1) if match else "instance"


def fun_kind(args: str) -> str:
    match = re.search(r"ty\s*=\s*([A-Za-z_][A-Za-z0-9_]*)", args)
    return match.group(1) if match else "fixed"


def normalize_identifier(name: str) -> str:
    name = name.strip()
    if name.startswith("r#"):
        name = name[2:]
    name = name.lstrip("_") or "arg"
    if keyword.iskeyword(name):
        name += "_"
    return name


def strip_obj_suffix(name: str) -> str:
    return name[:-3] if name.endswith("Obj") else name


def unwrap_generic(typ: str, generic: str) -> str | None:
    typ = typ.strip()
    prefix = f"{generic}<"
    if not typ.startswith(prefix) or not typ.endswith(">"):
        return None
    inner = typ[len(prefix) : -1]
    parts = split_top_level(inner)
    return parts[0] if parts else None


def normalize_rust_type(typ: str) -> str:
    typ = typ.strip()
    typ = re.sub(r"\bmut\s+", "", typ)
    typ = re.sub(r"&\s*(?:'static\s+)?", "", typ)
    typ = typ.strip()
    while (
        typ.startswith("(")
        and typ.endswith(")")
        and find_matching(typ, 0) == len(typ) - 1
    ):
        typ = typ[1:-1].strip()
    return typ


def python_type(typ: str | None, current_class: str | None = None) -> str:
    if typ is None:
        return "None"

    typ = normalize_rust_type(typ)
    result_inner = unwrap_generic(typ, "Result")
    if result_inner is not None:
        return python_type(result_inner, current_class)

    option_inner = unwrap_generic(typ, "Option")
    if option_inner is not None:
        return f"{python_type(option_inner, current_class)} | None"

    if typ in {"", "()"}:
        return "None"
    if typ == "Self":
        return strip_obj_suffix(current_class) if current_class else "Self"
    if typ in {"Obj", "Qstr", "ObjType", "Map", "InitToken"}:
        return "Any"
    if typ in {"i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize"}:
        return "int"
    if typ in {"f32", "f64"}:
        return "float"
    if typ == "bool":
        return "bool"
    if typ in {"str", "Str"}:
        return "str"
    if typ == "Callable":
        return "Callable[..., Any]"
    if typ.startswith("[") or typ.startswith("Vec<") or typ.startswith("Tuple<"):
        return "Any"
    if "<" in typ or ">" in typ or "'" in typ:
        return "Any"
    if typ.startswith("*"):
        return "Any"
    if re.fullmatch(r"Fun(?:\d+|Var|VarBetween|VarKw)?", typ):
        return "Callable[..., Any]"
    if "::" in typ:
        typ = typ.rsplit("::", 1)[1]
    return strip_obj_suffix(typ)


def collect_until(lines: list[str], start: int, delimiter: str) -> str:
    collected: list[str] = []
    for i in range(start, len(lines)):
        collected.append(lines[i].strip())
        if first_code_delimiter(lines[i], delimiter) != -1:
            break
    return " ".join(collected)


def collect_fn_signature(lines: list[str], start: int) -> str:
    collected: list[str] = []
    for i in range(start, len(lines)):
        line = lines[i]
        idx = first_code_delimiter(line, "{")
        if idx != -1:
            collected.append(line[:idx].strip())
            break
        collected.append(line.strip())
    return " ".join(collected)


def parse_rust_fn(text: str) -> RustFunction | None:
    match = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*\(", text)
    if not match:
        return None

    name = match.group(1)
    paren_start = text.find("(", match.start())
    paren_end = find_matching(text, paren_start)
    if paren_end == -1:
        return None

    params_text = text[paren_start + 1 : paren_end]
    params: list[RustParam] = []
    for raw_param in split_top_level(params_text):
        param = raw_param.strip()
        if not param:
            continue
        if param in {"self", "&self", "&mut self"}:
            params.append(RustParam("self", None, True))
            continue
        if ":" not in param:
            continue
        name_part, type_part = param.split(":", 1)
        name_part = name_part.strip()
        if name_part.startswith("mut "):
            name_part = name_part[4:].strip()
        params.append(RustParam(normalize_identifier(name_part), type_part.strip()))

    ret = None
    after = text[paren_end + 1 :].strip()
    if after.startswith("->"):
        ret = after[2:].strip()
        ret = re.split(r"\bwhere\b", ret, maxsplit=1)[0].strip()
    return RustFunction(name, params, ret)


def parse_const(text: str) -> tuple[str, str] | None:
    match = re.search(r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^=;]+)", text)
    if not match:
        return None
    return match.group(1), match.group(2).strip()


def normalize_signature(sig: str, default_return: str) -> str:
    sig = sig.strip()
    def_match = re.match(r"(?:async\s+)?def\s+[A-Za-z_][A-Za-z0-9_]*\s*(\(.*)", sig)
    if def_match:
        sig = def_match.group(1).strip()
    if not sig.startswith("("):
        sig = f"({sig})"

    close = find_matching(sig, 0)
    if close == -1:
        return sig

    tail = sig[close + 1 :].strip()
    if tail.startswith("->"):
        return sig
    if tail.startswith(":"):
        return f"{sig[: close + 1]} -> {tail[1:].strip()}"
    return f"{sig[: close + 1]} -> {default_return}"


def is_varargs_api(fn: RustFunction, attr: Attr | None) -> bool:
    if attr and fun_kind(attr.args) != "fixed":
        return True
    return any(param.typ and "[Obj]" in param.typ for param in fn.params)


def fallback_signature(
    fn: RustFunction,
    *,
    current_class: str | None = None,
    kind: str = "function",
    method_attr: Attr | None = None,
) -> str:
    if kind == "constructor":
        return "(self, *args: Any, **kwargs: Any) -> None"

    current_class = strip_obj_suffix(current_class) if current_class else None
    binding = binding_name(method_attr.args) if method_attr else "instance"
    params: list[str] = []

    if kind == "method":
        has_receiver = any(param.is_self for param in fn.params)
        if binding == "class":
            params.append("cls")
        elif binding != "static" and not has_receiver:
            params.append("self")

    if is_varargs_api(fn, method_attr):
        if not any(param.is_self for param in fn.params) or binding == "static":
            pass
        params.append("*args: Any")
        if method_attr and fun_kind(method_attr.args) == "kw":
            params.append("**kwargs: Any")
    else:
        manual_self_consumed = False
        for param in fn.params:
            if param.is_self:
                params.append("self")
                continue
            if param.typ is None:
                continue
            if (
                kind == "method"
                and binding == "instance"
                and not manual_self_consumed
                and not any(p.is_self for p in fn.params)
                and normalize_rust_type(param.typ) in {"Obj", "Self"}
            ):
                manual_self_consumed = True
                continue
            if param.typ.strip() == "&[Obj]":
                params.append("*args: Any")
                continue
            params.append(f"{param.name}: {python_type(param.typ, current_class)}")

    ret = python_type(fn.ret, current_class)
    return f"({', '.join(params)}) -> {ret}"


def callable_sigs(
    stub: StubMeta,
    fn: RustFunction,
    *,
    default_return: str,
    fallback: str,
) -> list[str]:
    if stub.sigs:
        return [normalize_signature(sig, default_return) for sig in stub.sigs]
    return [fallback]


def infer_constant_type(
    rust_type: str,
    text: str,
    current_class: str,
    source_id: str,
    classes_by_key: dict[tuple[str, str], ClassDef],
    classes_by_rust: dict[str, ClassDef],
) -> str:
    rust_type = normalize_rust_type(rust_type)
    if rust_type == "Self":
        return current_class
    if rust_type == "ObjType":
        target = re.search(r"=\s*([A-Za-z_][A-Za-z0-9_]*)::OBJ_TYPE", text)
        if target:
            target_name = target.group(1)
            cls = classes_by_key.get((source_id, target_name)) or classes_by_rust.get(
                target_name
            )
            return f"type[{cls.name if cls else strip_obj_suffix(target_name)}]"
        return "type[Any]"
    return python_type(rust_type, current_class)


def format_attr_line(attr: str) -> str:
    attr = attr.strip()
    if ":" in attr:
        return attr
    return f"{attr}: Any"


def scan_file(path: Path) -> ScanResult:
    lines = path.read_text().splitlines()
    source_id = str(path)
    result = ScanResult()
    pending_docs: list[str] = []
    pending_attrs: list[Attr] = []
    brace_depth = 0
    current_impl: ImplDef | None = None
    impl_item_depth: int | None = None

    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        depth_before = brace_depth
        relevant_depth = impl_item_depth if current_impl is not None else 0

        if relevant_depth is not None and depth_before == relevant_depth:
            if stripped.startswith("///"):
                pending_docs.append(stripped[3:].lstrip())
            elif stripped.startswith("#["):
                raw_attr, end = read_attribute(lines, i)
                pending_attrs.append(parse_attr(raw_attr))
                for attr_line in lines[i : end + 1]:
                    brace_depth += delimiter_delta(attr_line, "{", "}")
                i = end
                i += 1
                continue
            elif not stripped:
                pending_docs.clear()
                pending_attrs.clear()
            elif current_impl is None:
                class_attr = attr_named(pending_attrs, "class")
                fun_attr = attr_named(pending_attrs, "fun")
                class_methods_attr = attr_named(pending_attrs, "class_methods")

                if class_attr and re.search(
                    r"\bstruct\s+[A-Za-z_][A-Za-z0-9_]*", stripped
                ):
                    stub = parse_stub_meta(pending_attrs)
                    match = re.search(r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)", stripped)
                    py_name = qstr_name(class_attr.args)
                    if match and py_name and not stub.skip:
                        result.classes.append(
                            ClassDef(
                                rust_name=match.group(1),
                                name=py_name,
                                source_id=source_id,
                                docs=list(pending_docs),
                                attrs=list(stub.attrs),
                            )
                        )
                    pending_docs.clear()
                    pending_attrs.clear()
                elif fun_attr and re.search(r"\bfn\s+[A-Za-z_][A-Za-z0-9_]*", stripped):
                    stub = parse_stub_meta(pending_attrs)
                    rust_fn = parse_rust_fn(collect_fn_signature(lines, i))
                    if rust_fn and not stub.skip:
                        py_name = qstr_name(fun_attr.args) or rust_fn.name
                        fallback = fallback_signature(rust_fn, method_attr=fun_attr)
                        result.functions.append(
                            FunctionDef(
                                name=py_name,
                                sigs=callable_sigs(
                                    stub,
                                    rust_fn,
                                    default_return="Any",
                                    fallback=fallback,
                                ),
                                docs=list(pending_docs),
                            )
                        )
                    pending_docs.clear()
                    pending_attrs.clear()
                elif class_methods_attr:
                    match = re.search(
                        r"\bimpl\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", stripped
                    )
                    if match:
                        current_impl = ImplDef(match.group(1), source_id)
                        result.impls.append(current_impl)
                        impl_item_depth = depth_before + 1
                    pending_docs.clear()
                    pending_attrs.clear()
                else:
                    pending_docs.clear()
                    pending_attrs.clear()
            else:
                method_attr = attr_named(pending_attrs, "method")
                make_new_attr = attr_named(pending_attrs, "make_new")
                attr_attr = attr_named(pending_attrs, "attr")
                constant_attr = attr_named(pending_attrs, "constant")

                if (method_attr or make_new_attr or attr_attr) and re.search(
                    r"\bfn\s+[A-Za-z_][A-Za-z0-9_]*", stripped
                ):
                    stub = parse_stub_meta(pending_attrs)
                    rust_fn = parse_rust_fn(collect_fn_signature(lines, i))
                    if rust_fn and not stub.skip:
                        if attr_attr:
                            current_impl.attrs.extend(stub.attrs)
                        elif make_new_attr:
                            fallback = fallback_signature(
                                rust_fn,
                                current_class=current_impl.rust_type,
                                kind="constructor",
                                method_attr=make_new_attr,
                            )
                            current_impl.methods.append(
                                FunctionDef(
                                    name="__init__",
                                    sigs=callable_sigs(
                                        stub,
                                        rust_fn,
                                        default_return="None",
                                        fallback=fallback,
                                    ),
                                    docs=list(pending_docs),
                                )
                            )
                        elif method_attr:
                            binding = binding_name(method_attr.args)
                            decorators = []
                            if binding == "static":
                                decorators.append("staticmethod")
                            elif binding == "class":
                                decorators.append("classmethod")
                            fallback = fallback_signature(
                                rust_fn,
                                current_class=current_impl.rust_type,
                                kind="method",
                                method_attr=method_attr,
                            )
                            current_impl.methods.append(
                                FunctionDef(
                                    name=qstr_name(method_attr.args) or rust_fn.name,
                                    sigs=callable_sigs(
                                        stub,
                                        rust_fn,
                                        default_return="Any",
                                        fallback=fallback,
                                    ),
                                    docs=list(pending_docs),
                                    decorators=decorators,
                                )
                            )
                    pending_docs.clear()
                    pending_attrs.clear()
                elif constant_attr and re.search(
                    r"\bconst\s+[A-Za-z_][A-Za-z0-9_]*", stripped
                ):
                    stub = parse_stub_meta(pending_attrs)
                    const_text = collect_until(lines, i, ";")
                    parsed = parse_const(const_text)
                    if parsed and not stub.skip:
                        rust_name, rust_type = parsed
                        py_name = qstr_name(constant_attr.args) or rust_name
                        if "Fun" in rust_type and stub.typ is None:
                            sigs = [
                                normalize_signature(sig, "Any") for sig in stub.sigs
                            ] or ["(self, *args: Any) -> Any"]
                            current_impl.methods.append(
                                FunctionDef(py_name, sigs, list(pending_docs))
                            )
                        else:
                            current_impl.constants.append(
                                ConstantDef(
                                    name=py_name,
                                    typ=stub.typ or rust_type,
                                    docs=list(pending_docs),
                                    source=const_text,
                                )
                            )
                    pending_docs.clear()
                    pending_attrs.clear()
                else:
                    pending_docs.clear()
                    pending_attrs.clear()

        brace_depth += delimiter_delta(line, "{", "}")
        if (
            current_impl is not None
            and impl_item_depth is not None
            and brace_depth < impl_item_depth
        ):
            current_impl = None
            impl_item_depth = None
            pending_docs.clear()
            pending_attrs.clear()

        i += 1

    return result


def merge_results(results: Iterable[ScanResult]) -> ScanResult:
    merged = ScanResult()
    for result in results:
        merged.classes.extend(result.classes)
        merged.impls.extend(result.impls)
        merged.functions.extend(result.functions)
    return merged


def attach_impls(result: ScanResult) -> None:
    classes_by_key = {(cls.source_id, cls.rust_name): cls for cls in result.classes}
    classes_by_rust = {cls.rust_name: cls for cls in result.classes}
    for impl in result.impls:
        cls = classes_by_key.get((impl.source_id, impl.rust_type))
        if cls is None:
            continue
        cls.attrs.extend(impl.attrs)
        cls.methods.extend(impl.methods)
        for constant in impl.constants:
            constant.typ = infer_constant_type(
                constant.typ,
                constant.source,
                cls.name,
                impl.source_id,
                classes_by_key,
                classes_by_rust,
            )
            cls.constants.append(constant)


def docstring_lines(docs: list[str], indent: str) -> list[str]:
    if not docs:
        return []
    escaped = [line.replace('"""', r"\"\"\"") for line in docs]
    if len(escaped) == 1:
        return [f'{indent}"""{escaped[0]}"""']
    return [f'{indent}"""', *[f"{indent}{line}" for line in escaped], f'{indent}"""']


def render_function(fn: FunctionDef, indent: str = "") -> list[str]:
    lines: list[str] = []
    overload = len(fn.sigs) > 1
    for idx, sig in enumerate(fn.sigs):
        if overload:
            lines.append(f"{indent}@overload")
        for decorator in fn.decorators:
            lines.append(f"{indent}@{decorator}")
        header = f"{indent}def {fn.name}{sig}:"
        if idx == 0 and fn.docs:
            lines.append(header)
            lines.extend(docstring_lines(fn.docs, indent + "    "))
            lines.append(f"{indent}    ...")
        else:
            lines.append(f"{header} ...")
    return lines


def render_class(cls: ClassDef) -> list[str]:
    lines = [f"class {cls.name}:"]
    body: list[str] = []
    body.extend(docstring_lines(cls.docs, "    "))

    for attr in cls.attrs:
        body.append(f"    {format_attr_line(attr)}")

    for constant in cls.constants:
        body.append(f"    {constant.name}: ClassVar[{constant.typ}]")
        body.extend(docstring_lines(constant.docs, "    "))

    if cls.constants and cls.methods:
        body.append("")

    for idx, method in enumerate(cls.methods):
        if idx > 0:
            body.append("")
        body.extend(render_function(method, "    "))

    if not body:
        body.append("    ...")

    lines.extend(body)
    return lines


def render_stub(result: ScanResult) -> str:
    lines = [
        "# This file is generated by stubgen.py.",
        "from __future__ import annotations",
        "",
        "from typing import Any, Callable, ClassVar, overload",
        "",
    ]

    for idx, cls in enumerate(result.classes):
        if idx > 0:
            lines.append("")
        lines.extend(render_class(cls))

    if result.functions:
        if result.classes:
            lines.append("")
        for idx, fn in enumerate(result.functions):
            if idx > 0:
                lines.append("")
            lines.extend(render_function(fn))

    lines.append("")
    return "\n".join(lines)


def generate(src: Path) -> str:
    paths = sorted(src.rglob("*.rs"))
    result = merge_results(scan_file(path) for path in paths)
    attach_impls(result)
    return render_stub(result)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--src",
        type=Path,
        default=DEFAULT_SRC,
        help="Rust source directory to scan (default: src/modvenice)",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Write generated stubs to this file instead of stdout",
    )
    args = parser.parse_args()

    output = generate(args.src)
    if args.output:
        args.output.write_text(output)
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
