//! Role: Compare the committed schema files with the ones generated from the source.

use std::path::Path;

#[test]
fn committed_schemas_match_the_source() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let protocol =
        super::read::read_protocol(&manifest.join("../..")).expect("the protocol source reads");
    let files = [
        (
            "schema/kivact-devtools.schema.json",
            super::schema::schema(&protocol),
        ),
        (
            "schema/devtools-config.schema.json",
            super::schema::config_schema(&protocol),
        ),
    ];
    let write = std::env::var_os("KIVACT_WRITE_SCHEMA").is_some();
    for (name, expected) in files {
        let path = manifest.join(name);
        if write {
            std::fs::write(&path, &expected).expect("the schema file is writable");
            continue;
        }
        let current = std::fs::read_to_string(&path)
            .expect("the committed schema exists")
            .replace("\r\n", "\n");
        assert!(
            current == expected,
            "{name} is out of date; run `KIVACT_WRITE_SCHEMA=1 cargo test -p kivact-devtools-protocol --test schema_generated`"
        );
    }
}
