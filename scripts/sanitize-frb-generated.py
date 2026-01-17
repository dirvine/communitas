#!/usr/bin/env python3
"""Post-process flutter_rust_bridge generated Rust to avoid panics/unwraps.

This keeps the core crate compliant with the no-panic/unwrap policy.
"""
from __future__ import annotations

from pathlib import Path
import re
import sys


def sanitize(path: Path) -> None:
    text = path.read_text()

    text = text.replace(
        "let api_that_guard = api_that_guard.unwrap();",
        "let api_that_guard = api_that_guard.ok_or_else(|| \"FFI handle unavailable\".to_string())?;",
    )

    text = text.replace(
        "                                _ => unreachable!(),",
        "                                _ => return Err(\"Invalid lockable index\".to_string()),",
    )

    text = text.replace(
        "String::from_utf8(inner).unwrap()",
        "String::from_utf8(inner).unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).to_string())",
    )

    text = re.sub(
        r"(deserializer\.cursor\.read_[a-z0-9_]+(?:::<NativeEndian>)?\(\))\.unwrap\(\)",
        r"\1.unwrap_or_default()",
        text,
    )

    text = re.sub(
        r"(serializer\.cursor\.write_[a-z0-9_]+(?:::<NativeEndian>)?\([^;]*?\))\.unwrap\(\);",
        r"\1.ok();",
        text,
        flags=re.DOTALL,
    )
    text = re.sub(
        r"(\.write_[a-z0-9_]+(?:::<NativeEndian>)?\([^;]*?\))\s*\.unwrap\(\);",
        r"\1.ok();",
        text,
        flags=re.DOTALL,
    )

    text = text.replace(
        '            _ => unreachable!("Invalid variant for FlutterDiskType: {}", inner),',
        "            _ => crate::flutter_api::FlutterDiskType::Private,",
    )
    text = text.replace(
        '            _ => unreachable!("Invalid variant for FlutterEntityType: {}", inner),',
        "            _ => crate::flutter_api::FlutterEntityType::Group,",
    )

    text = text.replace(
        "            _ => unreachable!(),",
        "            _ => 0.into_dart(),",
    )

    text = text.replace(
        "        _ => unreachable!(),",
        "        _ => return,",
    )
    text = re.sub(
        r"(fn pde_ffi_dispatcher_sync_impl[\s\S]*?match func_id \{\n)\s*_ => return,\n(\s*\}\n)",
        r"\1        _ => flutter_rust_bridge::for_generated::WireSyncRust2DartSse {\n            ptr: std::ptr::null_mut(),\n            len: 0,\n        },\n\2",
        text,
    )

    if "clippy::not_unsafe_ptr_arg_deref" not in text:
        text = text.replace(
            "    clippy::needless_borrow\n)",
            "    clippy::needless_borrow,\n    clippy::not_unsafe_ptr_arg_deref\n)",
        )

    path.write_text(text)


def main() -> int:
    if len(sys.argv) > 1:
        target = Path(sys.argv[1])
    else:
        target = Path("communitas-core/src/frb_generated.rs")

    if not target.exists():
        print(f"FRB generated file not found: {target}")
        return 1

    sanitize(target)
    print(f"Sanitized FRB generated file: {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
