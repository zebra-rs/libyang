use libyang::*;

#[test]
fn leaf_test() {
    let mut yang = Yang::new();
    yang.add_path("/etc/openconfigd/yang:tests/yang/...");

    // Read a module.
    let mut ms = Modules::new();
    let data = yang.read(&ms, "test-leaf").unwrap();

    match yang_parse(&data) {
        Ok((_, module)) => {
            ms.modules.insert(module.prefix.to_owned(), module);

            let entry = ms.modules.get(&"test-leaf".to_string());
            if let Some(e) = entry {
                // Success.
                println!("Module found");
                println!("name: {}", e.name);
                println!("namespace: {}", e.namespace);
                println!("prefix: {}", e.prefix);
                for (_, t) in &e.typedefs {
                    println!("typedef: {}", t.name);
                }
                println!("Module dump: {:?}", e);
            } else {
                // Module not found.
                panic!("modules can't find")
            }
        }
        Err(e) => {
            panic!("module parse error: {:?}", e);
        }
    }
}
