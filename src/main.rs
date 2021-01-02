use libyang::*;

fn main() {
    // Allocate a new Yang.
    let mut yang = Yang::new();
    yang.add_path("/etc/openconfigd/yang:yang/...");

    // Read a module.
    let mut ms = Modules::new();
    let data = yang.read(&ms, "iana-if-type").unwrap();

    match yang_parse(&data) {
        Ok((_, module)) => {
            ms.modules.insert(module.prefix.to_owned(), module);

            let entry = ms.modules.get(&"ianaift".to_string());
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
        Err(e) => {
            println!("module parse: {:?}", e);
        }
    }
}
