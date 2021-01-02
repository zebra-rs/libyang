# libyang

`libyang` is a parser library for YANG (RFC6020, RFC7950) written in Rust
programming language.

## Example

``` rust
fn main() {
    // Allocate a new Yang.
    let mut yang = Yang::new();
    yang.add_path("/etc/openconfigd/yang:yang/...");

    // Read a module.
    match yang.read_and_parse("iana-if-type") {
        Ok(()) => {
            println!("Module read success");
        }
        Err(e) => {
            println!("Module parse error {:?}", e);
        }
    }

    let entry = yang.modules.get("ianaift");
    if let Some(e) = entry {
        println!("Module found");
        println!("name: {}", e.name);
        println!("namespace: {}", e.namespace);
        println!("prefix: {}", e.prefix);
        for (_, t) in &e.typedefs {
            println!("typedef: {}", t.name);
        }
        println!("Module dump: {:?}", e);
    } else {
        println!("Module not found")
    }
}
```

## License

Rocket is licensed under either of the following, at your option:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
 * MIT License ([LICENSE-MIT](LICENSE-MIT))
