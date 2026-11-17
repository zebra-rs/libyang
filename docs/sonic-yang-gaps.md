# Reading SONiC's YANG models — Design

SONiC ships a YANG model for its configuration tables: 146 modules in
`sonic-buildimage/src/sonic-yang-models/yang-models`. zebra-rs wants to load
them as the schema of a `sonic` configuration subtree (zebra-rs
`docs/design/sonic-junos-style-config.md`, phase 2). libyang 2.4.0 loaded none
of them.

This document lists what stood in the way, numbered 1 to 15, and what was
changed for each. 1 to 12 are forms the grammar refused. 13 to 15 are models
that parsed and gave a wrong tree.

## Result

| | 2.4.0 | With these changes |
|---|---|---|
| SONiC modules that parse | 0 of 146 | 146 |
| Modules that resolve their imports and build a tree | 0 | 146 |
| Lists with more than one key, read with all of them | 0 of 58 | 58 |
| Leaves left without a type | 5 | 0 |
| Leaves and union arms left as an unresolved typedef | 27 | 0 |

The tree of the 146 modules has 473 containers, 202 lists, 1,844 leaves and
65 leaf-lists.

## How the gaps were found

A probe parsed each module by itself and grouped the failures by the parser's
diagnostic. Each class was worked around on a scratch copy of the models (a
text rewrite), and the probe run again, until every file parsed. The same
modules were then loaded with their imports and built into `Entry` trees, and
the trees inspected: the keys of every list, the type of every leaf.

The counts below are of files that fail when that one workaround is left out
and all the others are in. A file can need several.

## 1 to 12: the grammar

### 1. A namespace that is not a URN — 146 files

```yang
namespace "http://github.com/sonic-net/sonic-vlan";
```

`UriArg` was `urn:[a-zA-Z0-9\-\.:]+`. RFC 7950 §7.1.3 makes the namespace a
URI, and outside the IETF it is usually an `http://` one.

**Change.** `UriArg` is a scheme, `:`, and the rest up to whitespace, `;` or
the closing quote.

**Found on the way.** `ModuleNode::namespace` held the token's debug form
(`'urn:…', Ty:92, at 53..87[…]`), not its text. It holds the text now.

### 2. `pattern`, `range` and `length` with substatements — 45 files

```yang
range "1..4095" {
    error-message "Invalid VLAN ID";
}
```

All three ended at `;`. RFC 7950 §9.2.4, §9.4.4 and §9.4.5 let each take
`error-message`, `error-app-tag`, `description` and `reference`, and
`pattern` also `modifier`.

**Change.** Each has a second form with a block of `RestrictionSubstmt`. The
`Range` scanner, which reads the argument of `range` and `length`, now also
ends at `{`.

**Not kept.** The substatements are parsed and dropped: `TypeNode` has no
field for an error message.

A pattern with `modifier invert-match` is one the value must *not* match
(§9.4.6). `TypeNode::pattern` is a pattern to match, so an inverted one is not
put there; it is dropped with its substatements. SONiC's models have none.

### 3. An unquoted leafref path — 10 files

```yang
path /port:sonic-port/port:PORT/port:PORT_LIST/port:name;
```

### 9. An unquoted pattern — 1 file

```yang
pattern [0-7];
```

Both arguments went through `Ystring`, whose unquoted form is `Identifier`: it
starts with a letter or `_`. A path starts with `/` or `..`, and a pattern with
anything.

**Change.** `pattern` and `path` switch to a scanner of their own, `Arg`. In
it an unquoted string is anything up to whitespace, `;`, `{`, `}` or a quote
(RFC 7950 §6.1.3), not starting with `+`. Quoted strings and `+` work in it as
elsewhere. The argument is the new non-terminal `ArgStr`.

A general unquoted string in the default scanner would have matched every
keyword; a scanner entered by these two keywords keeps the change to their
arguments.

An unquoted string there does not start with `//` or `/*`: the scanner
declares the two comment forms itself, so a comment between the keyword and
its argument is still a comment.

### 4. An extension used with no argument — 5 files

```yang
evtcmn:ALARM_SEVERITY_MINOR;
```

`UnknownStmt` required an argument. An extension has one only when its
definition has an `argument` statement (RFC 7950 §7.19).

**Change.** `UnknownStmt` has the form `IdentifierRef ;`. Its
`UnknownNode::argument` is then empty.

### 5. `error-app-tag` under `must` — 4 files

```yang
must "…" {
    error-message "…";
    error-app-tag voq-queue-local-port-missing;
}
```

**Change.** `must` takes `error-app-tag` and `reference` beside
`error-message` and `description` (RFC 7950 §7.5.3). Parsed and dropped, as
`error-message` was.

### 6. A typedef inside a container — 3 files

```yang
container sonic-crm {
    typedef threshold { … }
```

RFC 7950 §7.5.2 and §7.8.1 allow `typedef` in a container and in a list.

**Change.** Both take `TypedefStmt`. A type reference is looked up among the
module's typedefs, so `yang()` moves the nested ones there, after the module's
own (`hoist_typedefs`).

**Difference from the RFC.** §5.5 scopes a nested typedef to the node it is
written in. Here it is visible in the whole module. Every reference the RFC
allows resolves; a name defined both at module level and in a container
resolves to the module's, where the RFC would have the inner one shadow it.
`grouping` inside a container or list is still not read.

### 7. An empty default — 3 files

```yang
default "";
```

### 10. A default over several lines — 1 file

```yang
default "You are on
           ____   ___  _   _ _  ____
         …";
```

A quoted default was `"`, one token of `AsciiNoSemicolon`, `"`. That token is
at least one character and has no newline.

**Change.** A quoted default is a `DQString`, the double-quoted string used
everywhere else: empty, several lines, escapes. The `Default` scanner pushes
the string scanner on `"`. An unquoted default is read as before.

### 8. A decimal range — 1 file

```yang
type decimal64 {
    fraction-digits 2;
    range 0.0..99.99;
}
```

**Change.** A range bound (`RangeNumber`) may have a fraction.

The grammar then takes a fraction in every `range` and `length`. On an integer
type, and in a `length`, a bound that is not a number of the type is refused
while the file is parsed (`type_stmt_check`): the module fails to load with a
`ParseError` at the bound. That covers a fraction, and a bound the type cannot
hold (`range "0..300"` on a `uint8`), which used to panic.

**Not kept.** The range of a `decimal64` is not stored, as before: `RangeNode`
has integer variants only.

### 11. A backslash in a single-quoted string — 1 file

```yang
pattern '[^\n\r]+';
```

A backslash in a single-quoted string started an escape, and the character
after it was dropped from the text. `\n` and `\t` did not parse at all.
RFC 7950 §6.1.3: a single-quoted string has no escapes.

**Change.** In a single-quoted string a backslash is a character like any
other. `'\d{4}'` used to give `{4}` and now gives `\d{4}`.

### 12. Single- and double-quoted strings joined — 1 file

```yang
pattern '[^ @,\\' + "']*" {
    error-message '…';
}
```

Not a gap of its own: joining with `+` worked. This line needed 2 (the block)
and 11 (the backslashes before the closing quote). It is in the tests because
it is the hardest string in the models.

## 13 to 15: the tree

### 13. A list had only its first key — 58 lists

```yang
list VLAN_MEMBER_LIST {
    key "name port";
```

The grammar read every name in the key. `key()` in `nodes/ast.rs` took the
first and returned. `Entry::key` was `["name"]`.

**Change.** `key()` follows the names to the end. `Entry::key` is
`["name", "port"]`.

This was not particular to SONiC: zebra-rs's own models have 25 lists with
more than one key.

### 14. A typedef named with the module's own prefix — 5 leaves

```yang
module sonic-hash {
    prefix hash;
    …
    leaf-list ecmp_hash {
        type hash:hash-field;
```

A prefix was looked up among the imports only. A module's own prefix on
something it defines itself is valid (RFC 7950 §6.4.1), and the leaf was left
with no type.

**Change.** `prefix_resolve` answers the module's own name for its own prefix,
so a grouping, an identity or a feature named that way is found as well. A
typedef named that way is looked up among the typedefs of the module (or
submodule) the reference is written in.

An `identityref` whose `base` has the module's own prefix stays an
`identityref`, as one whose `base` has no prefix. Only a base in another
module makes it an enumeration of that module's identities.

### 15. A typedef of a typedef — 25 leaves, 2 union arms

```yang
typedef flex_delay_status {
    type stypes:boolean_type;
}
```

Resolving a typedef returned the type written in it. When that was another
typedef the leaf was left with `kind = Path`, the kind of a reference that has
not been resolved.

**Change.** `typedef_resolve` follows the chain to a built-in type. A typedef
defined by itself, through other typedefs or the arms of a union, has no type:
the reference that leads back resolves to nothing.

- **Where each step is looked up.** In the module that defines the typedef,
  with that module's prefixes. The arms of a union are looked up there too;
  they were looked up in the module that uses the union, which lost an arm
  written with no prefix.
- **The name the type is known by.** `TypeNode::typedef` is the last typedef
  before the built-in type, as it is written in the module that names it. A
  chain that ends `… -> inet:ipv4-address -> string` is known as
  `inet:ipv4-address`, which is what a consumer maps to an address type.
- **Restrictions.** A `pattern`, `length` or `range` written on a reference to
  a typedef replaces the one of the type under it. The reference may be in
  another typedef, on a leaf or leaf-list, or an arm of a union:
  `type stypes:interface_name { pattern "Vrf[a-zA-Z0-9_-]+"; }`. A pattern
  replaces the typedef's where RFC 7950 §9.4.5 has both hold: `TypeNode` has
  one pattern.
- **Fewer values of an enumeration.** `enum` statements on a reference name
  the values that are left (§9.6.4), as `hash:hash-field` does with 19 of the
  23 of `stypes:hash-field`.
- **A range on a typedef.** `type my-number { range "1..10"; }` is read before
  `my-number` is known to be a `uint8`. Its bounds are kept as `i64`, or as
  `u64` when one is too large for that, and given the type the chain ends at
  (`RangeNode::cast`). Such a range used to be dropped.

## Compatibility

What a consumer of 2.4.0 sees differently:

| Change | Effect |
|---|---|
| 13 | `Entry::key` and `KeyNode::keys` have every key of a list |
| 11 | A single-quoted string keeps its backslashes |
| 15 | A leaf whose type is a typedef of a typedef has the built-in kind, not `Path` |
| 15 | A `pattern`, `length`, `range` or `enum` written on a reference to a typedef is applied; it was dropped |
| 2 | A pattern with `modifier invert-match` parses, and is not in `TypeNode::pattern` |
| 15 | The arms of a union from another module are that module's |
| 1 | `ModuleNode::namespace` is the namespace, not a token dump |
| 6 | `ContainerNode` and `ListNode` have a `typedef` field (empty after parsing) |

zebra-rs's test suite passes against these changes unmodified (3,955 tests).

## Not done

These parse and are not in the `Entry` tree, so a consumer cannot use them
yet. They are the next step for the `sonic` subtree, not part of this work.

| What | Uses in SONiC's models | State |
|---|---|---|
| `must` | 281 | On the AST nodes; not on `Entry`; never evaluated |
| `when` | 131 | The same |
| `leafref` | 211 | The path is on `TypeNode`; the target is not checked |
| `description` | every node | Not on `Entry`: no source for help text |
| `default` | 314 | Parsed and dropped |
| `unique` | 10 | Parsed and dropped |
| `error-message`, `error-app-tag` | 259 | Parsed and dropped |

Smaller things seen and left:

- An escape in a double-quoted string (`\n`, `\"`, `\\`) is dropped from the
  text, not translated.
- A union written directly inside a union is not flattened into its arms.
- A single-quoted default is read as an unquoted one, quotes included. It
  parses; defaults are not kept, so nothing sees the difference yet.

## Testing

- `tests/sonic_gaps.rs`: one test for each number above, on the fixtures
  `tests/yang/gap-syntax.yang`, `gap-types.yang` and `gap-ns-*.yang`.
- `tests/type_resolution.rs`: restrictions on a reference to a typedef,
  typedefs defined by themselves, an `identityref` by the module's own prefix,
  and the bounds of a `range` and a `length`.
- The 146 SONiC modules, unmodified, parse, resolve and build a tree with no
  diagnostic. They are not in this repository; the probe that loads them is
  not either.
- zebra-rs's unit suite against this tree as a path dependency.
