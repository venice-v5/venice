#!/usr/bin/env python3
"""Audit Python-facing documentation generated from Venice Rust doc comments."""

from __future__ import annotations

import argparse
import ast
import re
import subprocess
import sys
import textwrap
from collections import defaultdict
from pathlib import Path

PYTHON_LINK_RE = re.compile(r"`([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)+)`")
RUST_RESIDUE = {
    "Rust path separator (`::`)": re.compile(r"::"),
    "Rust fenced-code marker": re.compile(r"```(?:no_run|rust)\b"),
    "vexide path": re.compile(r"\bvexide(?:::|\.)"),
    "Rust binding/result syntax": re.compile(
        r"(?m)(?:^\s*let\s+(?:mut\s+)?[A-Za-z_]|\b(?:Ok\(|Err\(|unwrap\(|peripherals\.))"
    ),
}


def find_repo(start: Path) -> Path:
    for candidate in (start, *start.parents):
        if (candidate / "packages/venice/stubgen.py").is_file():
            return candidate
    raise SystemExit("could not find the Venice repository; pass --repo")


def generate_stub(repo: Path) -> str:
    command = [sys.executable, str(repo / "packages/venice/stubgen.py")]
    result = subprocess.run(command, cwd=repo, text=True, capture_output=True)
    if result.returncode:
        sys.stderr.write(result.stderr)
        raise SystemExit(f"stub generation failed with exit code {result.returncode}")
    return result.stdout


def following_string(body: list[ast.stmt], index: int) -> str | None:
    if index + 1 >= len(body):
        return None
    node = body[index + 1]
    if isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
        return node.value.value
    return None


def is_classvar(annotation: ast.expr) -> bool:
    return (
        isinstance(annotation, ast.Subscript)
        and isinstance(annotation.value, ast.Name)
        and annotation.value.id == "ClassVar"
    )


def function_parameters(node: ast.FunctionDef | ast.AsyncFunctionDef) -> list[str]:
    args = [*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs]
    names = [arg.arg for arg in args if arg.arg not in {"self", "cls"}]
    if node.args.vararg:
        names.append(node.args.vararg.arg)
    if node.args.kwarg:
        names.append(node.args.kwarg.arg)
    return names


def source_files(repo: Path, requested: list[Path]) -> list[Path]:
    paths: set[Path] = set()
    for raw in requested:
        path = raw if raw.is_absolute() else repo / raw
        if path.is_dir():
            paths.update(path.rglob("*.rs"))
        elif path.is_file() and path.suffix == ".rs":
            paths.add(path)
        elif path.is_file():
            raise SystemExit(f"source file is not Rust: {raw}")
        else:
            raise SystemExit(f"source path does not exist: {raw}")
    if not paths:
        raise SystemExit("the requested --source paths contain no Rust files")
    return sorted(paths)


def source_python_examples(path: Path) -> list[tuple[int, str, bool]]:
    examples: list[tuple[int, str, bool]] = []
    current: list[str] | None = None
    start_line = 0
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        match = re.match(r"^\s*///(?: ?)(.*)$", line)
        if not match:
            if current is not None:
                examples.append((start_line, "\n".join(current), False))
                current = None
            continue
        body = match.group(1)
        if current is None:
            if body.strip().lower() == "```python":
                current = []
                start_line = line_number + 1
        elif body.strip() == "```":
            examples.append((start_line, "\n".join(current), True))
            current = None
        else:
            current.append(body)
    if current is not None:
        examples.append((start_line, "\n".join(current), False))
    return examples


def collect(tree: ast.Module):
    docs: dict[str, list[str]] = defaultdict(list)
    params: dict[str, set[str]] = defaultdict(set)
    required: set[str] = set()
    class_members: dict[str, set[str]] = defaultdict(set)
    attributes: dict[str, set[str]] = defaultdict(set)

    for node in tree.body:
        if isinstance(node, ast.ClassDef):
            class_name = node.name
            required.add(class_name)
            class_members[class_name]
            class_doc = ast.get_docstring(node, clean=False)
            if class_doc:
                docs[class_name].append(class_doc)

            for index, item in enumerate(node.body):
                if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    qname = f"{class_name}.{item.name}"
                    required.add(qname)
                    class_members[class_name].add(item.name)
                    params[qname].update(function_parameters(item))
                    doc = ast.get_docstring(item, clean=False)
                    if doc:
                        docs[qname].append(doc)
                elif isinstance(item, ast.AnnAssign) and isinstance(item.target, ast.Name):
                    name = item.target.id
                    class_members[class_name].add(name)
                    if is_classvar(item.annotation):
                        qname = f"{class_name}.{name}"
                        required.add(qname)
                        doc = following_string(node.body, index)
                        if doc:
                            docs[qname].append(doc)
                    else:
                        attributes[class_name].add(name)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            required.add(node.name)
            params[node.name].update(function_parameters(node))
            doc = ast.get_docstring(node, clean=False)
            if doc:
                docs[node.name].append(doc)

    return docs, params, required, class_members, attributes


def select(names: set[str], matcher: re.Pattern[str] | None) -> set[str]:
    if matcher is None:
        return names
    return {name for name in names if matcher.search(name)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, help="Venice repository root")
    parser.add_argument(
        "--match",
        help="audit only qualified API names matching this regular expression",
    )
    parser.add_argument(
        "--source",
        action="append",
        type=Path,
        default=[],
        help="Rust source file or directory whose Python fences should be syntax-checked; repeatable",
    )
    parser.add_argument(
        "--skip-arguments",
        action="store_true",
        help="skip the heuristic that every parameter name appears in its docstring",
    )
    args = parser.parse_args()
    if not args.source:
        parser.error("at least one --source file or directory is required for example validation")

    repo = (args.repo or find_repo(Path.cwd())).resolve()
    matcher = re.compile(args.match) if args.match else None
    stub = generate_stub(repo)

    try:
        tree = ast.parse(stub)
    except SyntaxError as error:
        print(f"generated stub is invalid Python: {error}")
        return 1

    docs, params, required, class_members, attributes = collect(tree)
    selected = select(required, matcher)
    issues: list[str] = []
    if matcher is not None and not selected:
        issues.append(f"--match pattern selected no generated APIs: {args.match}")

    for qname in sorted(selected):
        if not docs.get(qname):
            issues.append(f"{qname}: missing docstring")

    if not args.skip_arguments:
        for qname in sorted(selected):
            if not docs.get(qname):
                continue
            text = "\n".join(docs[qname])
            for parameter in sorted(params[qname]):
                if not re.search(rf"\b{re.escape(parameter)}\b", text):
                    issues.append(f"{qname}: parameter `{parameter}` is not documented")

    selected_classes = {name for name in selected if name in class_members}
    for class_name in sorted(selected_classes):
        text = "\n".join(docs.get(class_name, []))
        if not text:
            continue
        for attribute in sorted(attributes[class_name]):
            if not re.search(rf"\b{re.escape(attribute)}\b", text):
                issues.append(f"{class_name}: attribute `{attribute}` is not documented in the class docstring")

    docs_to_check: dict[str, str] = {}
    for qname in selected:
        if docs.get(qname):
            docs_to_check[qname] = "\n".join(docs[qname])

    known_classes = set(class_members)
    known_modules = {"venice", "battery", "display", "vasyncio"}
    for qname, text in sorted(docs_to_check.items()):
        for link in PYTHON_LINK_RE.findall(text):
            parts = link.split(".")
            owner = parts[0]
            valid = False
            if owner in known_classes:
                valid = len(parts) == 2 and parts[1] in class_members[owner]
            elif owner in known_modules:
                if len(parts) == 2:
                    valid = parts[1] in required
                elif len(parts) == 3 and parts[1] in known_classes:
                    valid = parts[2] in class_members[parts[1]]
            if not valid:
                issues.append(f"{qname}: invalid or unresolvable Python API link `{link}`")

        for label, pattern in RUST_RESIDUE.items():
            if pattern.search(text):
                issues.append(f"{qname}: contains {label}")

        if re.search(r"(?m)^# (?:Errors|Exceptions)\s*$", text):
            issues.append(f"{qname}: use the Python-facing `# Raises` heading")

        if re.search(
            r'''(?mi)^\s*print\(\s*(?!(?:fr|rf|f)["'])(?:r|u|b|br|rb)?["'][^\n]*\{[^\n}]+\}''',
            text,
        ):
            issues.append(f"{qname}: possible non-f-string containing braces in an example")

    for path in source_files(repo, args.source):
        for start_line, example, closed in source_python_examples(path):
            relative = path.relative_to(repo) if path.is_relative_to(repo) else path
            if not closed:
                issues.append(f"{relative}:{start_line - 1}: unterminated Python code fence")
                continue
            try:
                ast.parse(textwrap.dedent(example))
            except SyntaxError as error:
                line = start_line + (error.lineno or 1) - 1
                issues.append(
                    f"{relative}:{line}: Python example has invalid syntax: {error.msg}"
                )

    if issues:
        print(f"documentation audit failed with {len(issues)} issue(s):")
        for issue in issues:
            print(f"- {issue}")
        return 1

    scope = args.match or "all generated APIs"
    print(f"documentation audit passed for {scope}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
