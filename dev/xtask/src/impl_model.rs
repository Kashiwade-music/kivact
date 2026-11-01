//! Role: Enforce the engine's declared Model processing modules syntactically.
//!
//! This intentionally approximate check recognizes inherent implementations whose
//! self type ends in `Model`; renamed type aliases are left to module analysis.
//! It follows declared modules, including inline modules and explicit #[path],
//! rather than assuming that a filename identifies its Rust module.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// model::interact is part of the Model processing tier, not a second root.
const ALLOWED: &[&str] = &["model", "paint", "a11y", "check"];

pub(crate) fn check(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let mut visited = BTreeSet::new();
    let file = root.join("crates/kivact-engine/src/lib.rs");
    walk_file(&file, "", &mut visited, &mut errors);
    errors
}

fn walk_file(
    file: &Path,
    module: &str,
    visited: &mut BTreeSet<(PathBuf, String)>,
    errors: &mut Vec<String>,
) {
    if !visited.insert((file.to_path_buf(), module.to_owned())) {
        return;
    }
    let parsed = std::fs::read_to_string(file)
        .map_err(|error| error.to_string())
        .and_then(|text| syn::parse_file(&text).map_err(|error| error.to_string()));
    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(format!("impl Model: {}: {error}", file.display()));
            return;
        }
    };
    let parent = file.parent().expect("module source has a parent");
    let stem = file.file_stem().expect("module source has a stem");
    let default_dir = if matches!(stem.to_str(), Some("lib" | "main" | "mod")) {
        parent.to_path_buf()
    } else {
        parent.join(stem)
    };
    walk_items(
        &parsed.items,
        file,
        parent,
        &default_dir,
        module,
        visited,
        errors,
    );
}

fn walk_items(
    items: &[syn::Item],
    file: &Path,
    attribute_dir: &Path,
    default_dir: &Path,
    module: &str,
    visited: &mut BTreeSet<(PathBuf, String)>,
    errors: &mut Vec<String>,
) {
    for item in items {
        match item {
            syn::Item::Impl(item) if item.trait_.is_none() => {
                let syn::Type::Path(path) = item.self_ty.as_ref() else {
                    continue;
                };
                if !path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "Model")
                {
                    continue;
                }
                let root = module.split("::").next().unwrap_or(module);
                if !ALLOWED.contains(&root) || module.split("::").any(|part| part == "state") {
                    errors.push(format!(
                        "impl Model: {} ({module}) is not a processing module",
                        file.display()
                    ));
                }
            }
            syn::Item::Mod(item) => {
                let name = item.ident.to_string();
                let child = if module.is_empty() {
                    name.clone()
                } else {
                    format!("{module}::{name}")
                };
                if let Some((_, items)) = &item.content {
                    walk_items(
                        items,
                        file,
                        &attribute_dir.join(&name),
                        &default_dir.join(&name),
                        &child,
                        visited,
                        errors,
                    );
                    continue;
                }
                let mut explicit = None;
                for attribute in &item.attrs {
                    if attribute.path().is_ident("path") {
                        if let syn::Meta::NameValue(value) = &attribute.meta
                            && let syn::Expr::Lit(value) = &value.value
                            && let syn::Lit::Str(value) = &value.lit
                        {
                            explicit = Some(attribute_dir.join(value.value()));
                        }
                        if explicit.is_none() {
                            errors.push(format!(
                                "impl Model: {}: unsupported #[path] for {child}",
                                file.display()
                            ));
                        }
                    }
                }
                let path = if let Some(path) = explicit {
                    path
                } else {
                    let flat = default_dir.join(format!("{name}.rs"));
                    let nested = default_dir.join(&name).join("mod.rs");
                    match (flat.exists(), nested.exists()) {
                        (true, false) => flat,
                        (false, true) => nested,
                        _ => {
                            errors.push(format!(
                                "impl Model: {}: missing or ambiguous source for {child}",
                                file.display()
                            ));
                            continue;
                        }
                    }
                };
                walk_file(&path, &child, visited, errors);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Verify declared Model processing boundaries with parsed source trees.

    use super::check;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Exercise the normal checker entry point over a disposable declared tree.
    // The snippets are parsed policy inputs, never compiled or linked as crates.
    struct SourceTree {
        root: PathBuf,
    }

    impl SourceTree {
        fn new(source: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "kivact-impl-model-{}-{nonce}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(root.join("crates/kivact-engine/src"))
                .expect("temporary source directory");
            let tree = Self { root };
            tree.write("lib.rs", source);
            tree
        }

        fn write(&self, name: &str, source: &str) {
            std::fs::write(
                self.root.join("crates/kivact-engine/src").join(name),
                source,
            )
            .expect("temporary policy source");
        }
    }

    impl Drop for SourceTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn rejects_inherent_model_processing_in_tree_and_host() {
        let tree = SourceTree::new(
            "struct Model; mod tree { impl super::Model {} } mod host { impl super::Model {} }",
        );
        let errors = check(&tree.root);
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(
            errors.iter().any(|error| error.contains("(tree)")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|error| error.contains("(host)")),
            "{errors:?}"
        );
    }

    #[test]
    fn accepts_processing_roots_and_interact_but_rejects_passive_state() {
        let tree = SourceTree::new(
            "struct Model;
             mod model { impl super::Model {} mod interact { impl crate::Model {} } }
             mod paint { impl super::Model {} }
             mod a11y { impl super::Model {} }
             mod check { impl super::Model {} }",
        );
        assert!(check(&tree.root).is_empty());
        tree.write(
            "lib.rs",
            "struct Model; mod model { mod focus { mod state { impl crate::Model {} } } }",
        );
        let errors = check(&tree.root);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("(model::focus::state)"), "{errors:?}");
    }

    #[test]
    fn ignores_comment_and_string_text_and_documented_trait_implementations() {
        let tree = SourceTree::new(
            r##"struct Model;
                mod host {
                    // impl Model { fn forbidden() {} }
                    /* impl crate::Model {} */
                    const TEXT: &str = r#"impl Model { fn forbidden() {} }"#;
                    impl Drop for super::Model { fn drop(&mut self) {} }
                    struct HostModel;
                    impl HostModel {}
                }"##,
        );
        let errors = check(&tree.root);
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn uses_declared_module_identity_when_filenames_differ() {
        let tree =
            SourceTree::new("struct Model; #[path = \"misleading_model.rs\"] mod host; mod model;");
        tree.write("misleading_model.rs", "impl crate::Model {}");
        tree.write("model.rs", "mod interact { impl crate::Model {} }");
        let errors = check(&tree.root);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("(host)"), "{errors:?}");
    }
}
