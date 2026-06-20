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

## Installation

Add it to your `Cargo.toml`:

```toml
[dependencies]
libyang = "1"
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

## How it works

The pipeline runs in four stages:

1. **Parse** — YANG text is parsed by the `parol`-generated grammar into a `YangGrammar` AST.
2. **AST** — `yang()` converts the grammar tree into structured `Node` values.
3. **Resolve** — `YangStore` loads imports/includes, merges submodules, and resolves typedefs, groupings, and identities.
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
