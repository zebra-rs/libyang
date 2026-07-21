# libyang

A YANG parser and data-modeling library written in Rust.

[YANG](https://datatracker.ietf.org/doc/html/rfc7950) (RFC 7950) is the data
modeling language used to describe configuration and state data for network
management protocols such as NETCONF and RESTCONF. `libyang` parses YANG
modules, resolves their dependencies, and turns them into a tree you can walk
to drive configuration and validation.

## Features

- RFC 7950 grammar, parsed with a [`parol`](https://crates.io/crates/parol)-generated parser.
- Module loading with automatic `import` / `include` (submodule) resolution.
- `typedef`, `grouping`, `identity`, and `union` resolution.
- An `Entry` tree suitable for building config schemas and validators.
- Deterministic output: the tree has the same shape on every run, so it can be
  diffed or used to generate golden files.
- Errors and schema problems are returned, never printed.

## Installation

Add it to your `Cargo.toml`:

```toml
[dependencies]
libyang = "2"
```

## Usage

Load a module by name from a search path, resolve its dependencies, and build
the `Entry` tree:

```rust
use libyang::{to_entry, YangStore};

let mut store = YangStore::new();
store.add_path("yang"); // directory containing your .yang files

// Parse the module and everything it imports/includes.
store.read_with_resolve("ietf-bgp").expect("parse and resolve");
store.identity_resolve();

let module = store.find_module("ietf-bgp").expect("module found");
let entry = to_entry(&store, module);

println!("loaded module: {}", entry.name);
```

For low-level access, you can parse a single file into the grammar AST directly:

```rust
use libyang::yang;
use libyang::yang_grammar::YangGrammar;
use libyang::yang_parser::parse;

let input = std::fs::read_to_string("yang/ietf-bgp@2023-07-05.yang").unwrap();
let mut grammar = YangGrammar::new();
parse(&input, "ietf-bgp", &mut grammar).expect("parse");

let node = yang(grammar).expect("build node tree");
```

## Diagnostics

A malformed `augment` — a target that does not resolve, one that lands on a
leaf, or one that introduces a name the target already has — does not stop the
build: the tree is still produced with that augment skipped. Those findings are
collected on the store rather than written to stderr, so the caller decides
whether to log them, fail, or ignore them:

```rust
use libyang::{to_entry, YangStore};

let mut store = YangStore::new();
store.add_path("yang");
store.read_with_resolve("ietf-bgp").expect("parse and resolve");
store.identity_resolve();

let module = store.find_module("ietf-bgp").expect("module found");
let entry = to_entry(&store, module);

for diagnostic in store.take_diagnostics() {
    eprintln!("warning: {diagnostic}");
}
```

Each `Diagnostic` names the module at fault along with the target and the
offending node, and can be matched on rather than parsed.

## Migrating from 1.x

2.0 is a breaking release:

- `YangStore::read` is gone. Use `read_with_resolve`, which loads each module
  once (so mutually importing modules terminate) and keeps submodules.
- `YangStore::modules` and `submodules` are private. Use `find_module` /
  `find_submodule`.
- `YangError`'s variants carry the file, module and underlying parse
  diagnostic, and the enum is `#[non_exhaustive]`. Parse failures are returned
  instead of being printed to stdout.
- Augment problems are collected as `Diagnostic`s (see above) instead of being
  written to stderr.
- `RangeNode` implements `Display` instead of having an inherent `to_string`;
  `.to_string()` still works.

## How it works

The pipeline runs in four stages:

1. **Parse** — YANG text is parsed by the `parol`-generated grammar into a `YangGrammar` AST.
2. **AST** — `yang()` converts the grammar tree into structured `Node` values.
3. **Resolve** — `YangStore` loads imports and includes, keeping submodules addressable through `find_submodule`, and resolves typedefs, groupings, and identities. Modules that import each other are loaded once, so cycles terminate.
4. **Entry** — `to_entry()` produces an `Entry` tree for data modeling and validation.

## Testing

```sh
cargo test
```

Integration tests live in `tests/` and run against fixture modules in
`tests/yang/` and the standard modules in `yang/`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
