use libyang::*;

#[test]
fn leaf_test() {
    let mut yang = Yang::new();
    yang.add_path("/etc/openconfigd/yang:tests/yang/...");

    // Read a module.
    let mut ms = Modules::new();
    let yang_name = "test-leaf";
    let data = yang.read(&ms, yang_name).unwrap();

    match yang_parse(&data) {
        Ok((_, module)) => {
            ms.modules.insert(module.prefix.to_owned(), module);

            let entry = ms.modules.get(&"test-leaf".to_string());
            if let Some(_) = entry {
                // Success.
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
