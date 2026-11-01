//! Role: Resolve explicitly exported constructors to their Rust source definitions.
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct Module {
    pub(crate) items: Vec<syn::Item>,
    directory: PathBuf,
    pub(crate) path: Vec<String>,
    pub(crate) crate_name: String,
}

fn read(file: &Path, crate_name: &str, path: Vec<String>) -> Module {
    let source = std::fs::read_to_string(file)
        .unwrap_or_else(|error| panic!("read {}: {error}", file.display()));
    Module {
        items: syn::parse_file(&source).expect("valid Rust source").items,
        directory: if file.file_name().is_some_and(|name| name == "lib.rs") {
            file.parent().expect("crate root parent").to_path_buf()
        } else {
            file.with_extension("")
        },
        path,
        crate_name: crate_name.to_owned(),
    }
}

pub(crate) fn crate_root(root: &Path, name: &str) -> Module {
    read(
        &root
            .join("crates")
            .join(name.replace('_', "-"))
            .join("src/lib.rs"),
        name,
        Vec::new(),
    )
}

fn child(module: &Module, item: &syn::ItemMod) -> Module {
    let mut path = module.path.clone();
    path.push(item.ident.to_string());
    if let Some((_, items)) = &item.content {
        Module {
            items: items.clone(),
            directory: module.directory.join(item.ident.to_string()),
            path,
            crate_name: module.crate_name.clone(),
        }
    } else {
        assert!(
            !item.attrs.iter().any(|attr| attr.path().is_ident("path")),
            "constructor resolver needs support for #[path] modules"
        );
        read(
            &module.directory.join(format!("{}.rs", item.ident)),
            &module.crate_name,
            path,
        )
    }
}

pub(crate) fn uses(
    tree: &syn::UseTree,
    prefix: &[String],
    result: &mut Vec<(Vec<String>, String)>,
) {
    match tree {
        syn::UseTree::Path(item) => {
            let mut prefix = prefix.to_vec();
            prefix.push(item.ident.to_string());
            uses(&item.tree, &prefix, result);
        }
        syn::UseTree::Name(item) => {
            let mut path = prefix.to_vec();
            if item.ident != "self" {
                path.push(item.ident.to_string());
            }
            let name = path.last().expect("use names a path").clone();
            result.push((path, name));
        }
        syn::UseTree::Rename(item) => {
            let mut path = prefix.to_vec();
            if item.ident != "self" {
                path.push(item.ident.to_string());
            }
            result.push((path, item.rename.to_string()));
        }
        syn::UseTree::Group(item) => {
            for item in &item.items {
                uses(item, prefix, result);
            }
        }
        syn::UseTree::Glob(_) => result.push((prefix.to_vec(), "*".into())),
    }
}

fn module_at(root: &Path, module: &Module, path: &[String], depth: usize) -> Module {
    assert!(
        depth < 64,
        "constructor module export cycle at {}",
        path.join("::")
    );
    let mut full = module.path.clone();
    let mut rest = path;
    let crate_name = match rest.first().map(String::as_str) {
        Some("crate") => {
            full.clear();
            rest = &rest[1..];
            module.crate_name.as_str()
        }
        Some(name) if name.starts_with("kivact") => {
            full.clear();
            rest = &rest[1..];
            name
        }
        _ => module.crate_name.as_str(),
    };
    while let Some(first) = rest.first() {
        match first.as_str() {
            "self" => rest = &rest[1..],
            "super" => {
                full.pop().expect("super has a parent");
                rest = &rest[1..];
            }
            _ => break,
        }
    }
    full.extend_from_slice(rest);
    descend(root, crate_root(root, crate_name), &full, depth)
}

fn descend(root: &Path, module: Module, path: &[String], depth: usize) -> Module {
    let Some((name, rest)) = path.split_first() else {
        return module;
    };
    for item in &module.items {
        match item {
            syn::Item::Mod(item) if item.ident == name.as_str() => {
                return descend(root, child(&module, item), rest, depth);
            }
            syn::Item::Use(item) => {
                let mut leaves = Vec::new();
                uses(&item.tree, &[], &mut leaves);
                for (source, exposed) in leaves {
                    if exposed == *name {
                        let target = module_at(root, &module, &source, depth + 1);
                        return descend(root, target, rest, depth + 1);
                    }
                }
            }
            _ => {}
        }
    }
    panic!("{}: no module {}", module.crate_name, name)
}

fn source(
    root: &Path,
    module: &Module,
    path: &[String],
    depth: usize,
) -> Option<(syn::Signature, Module)> {
    assert!(
        depth < 64,
        "constructor export cycle at {}",
        path.join("::")
    );
    let (name, parents) = path.split_last().expect("export names a constructor");
    let target = module_at(root, module, parents, depth);
    exported(root, &target, name, depth + 1)
}
fn exported(
    root: &Path,
    module: &Module,
    name: &str,
    depth: usize,
) -> Option<(syn::Signature, Module)> {
    for item in &module.items {
        match item {
            syn::Item::Fn(item)
                if item.sig.ident == name && matches!(item.vis, syn::Visibility::Public(_)) =>
            {
                return Some((item.sig.clone(), module.clone()));
            }
            syn::Item::Use(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                let mut leaves = Vec::new();
                uses(&item.tree, &[], &mut leaves);
                for (path, exposed) in leaves {
                    if exposed == name {
                        return source(root, module, &path, depth);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn constructor(
    root: &Path,
    crate_name: &str,
    name: &str,
) -> Option<(syn::Signature, Module)> {
    exported(root, &crate_root(root, crate_name), name, 0)
}
