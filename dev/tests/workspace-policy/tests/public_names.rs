//! Role: Golden inventory of names exposed by every public facade module.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct Module {
    items: Vec<syn::Item>,
    directory: PathBuf,
    path: Vec<String>,
    crate_name: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root is three levels above this crate")
        .to_path_buf()
}

fn read_module(file: &Path, crate_name: &str, path: Vec<String>) -> Module {
    let source = std::fs::read_to_string(file)
        .unwrap_or_else(|error| panic!("read {}: {error}", file.display()));
    let parsed = syn::parse_file(&source).expect("public module is valid Rust");
    let directory = if file.file_name().is_some_and(|name| name == "lib.rs") {
        file.parent()
            .expect("crate root has a parent")
            .to_path_buf()
    } else {
        file.with_extension("")
    };
    Module {
        items: parsed.items,
        directory,
        path,
        crate_name: crate_name.to_owned(),
    }
}

fn crate_root(root: &Path, name: &str) -> Module {
    read_module(
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
            "module path attributes need explicit inventory support"
        );
        read_module(
            &module.directory.join(format!("{}.rs", item.ident)),
            &module.crate_name,
            path,
        )
    }
}

/// Resolve use trees to source paths and the names actually visible to consumers.
fn uses(tree: &syn::UseTree, prefix: &[String], result: &mut Vec<(Vec<String>, String)>) {
    match tree {
        syn::UseTree::Path(item) => {
            let mut prefix = prefix.to_vec();
            prefix.push(item.ident.to_string());
            uses(&item.tree, &prefix, result);
        }
        syn::UseTree::Name(item) => {
            let name = item.ident.to_string();
            let mut path = prefix.to_vec();
            if name != "self" {
                path.push(name.clone());
            }
            let exposed = if name == "self" {
                prefix.last().expect("self has a prefix").clone()
            } else {
                name
            };
            result.push((path, exposed));
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
        syn::UseTree::Glob(_) => result.push((prefix.to_vec(), "*".to_owned())),
    }
}

fn resolve(root: &Path, context: &Module, path: &[String]) -> Module {
    let (first, rest) = path.split_first().expect("module path is nonempty");
    match first.as_str() {
        "crate" => descend(root, crate_root(root, &context.crate_name), rest),
        "self" => descend(root, context.clone(), rest),
        "super" => {
            let parent = &context.path[..context.path.len() - 1];
            let parent = descend(root, crate_root(root, &context.crate_name), parent);
            resolve(root, &parent, rest)
        }
        name if name.starts_with("kivact") => descend(root, crate_root(root, name), rest),
        _ => descend(root, context.clone(), path),
    }
}

fn descend(root: &Path, module: Module, path: &[String]) -> Module {
    let Some((name, rest)) = path.split_first() else {
        return module;
    };
    for item in &module.items {
        if let syn::Item::Mod(item) = item
            && item.ident == name.as_str()
        {
            return descend(root, child(&module, item), rest);
        }
        if let syn::Item::Use(item) = item {
            let mut leaves = Vec::new();
            uses(&item.tree, &[], &mut leaves);
            for (source, exposed) in leaves {
                if exposed == *name {
                    return descend(root, resolve(root, &module, &source), rest);
                }
            }
        }
    }
    panic!("module {} has no module {name}", module.path.join("::"));
}

fn names(root: &Path, module: &Module) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for item in &module.items {
        let declaration = match item {
            syn::Item::Const(item) => Some((&item.vis, &item.ident)),
            syn::Item::Enum(item) => Some((&item.vis, &item.ident)),
            syn::Item::Fn(item) => Some((&item.vis, &item.sig.ident)),
            syn::Item::Mod(item) => Some((&item.vis, &item.ident)),
            syn::Item::Static(item) => Some((&item.vis, &item.ident)),
            syn::Item::Struct(item) => Some((&item.vis, &item.ident)),
            syn::Item::Trait(item) => Some((&item.vis, &item.ident)),
            syn::Item::TraitAlias(item) => Some((&item.vis, &item.ident)),
            syn::Item::Type(item) => Some((&item.vis, &item.ident)),
            syn::Item::Union(item) => Some((&item.vis, &item.ident)),
            syn::Item::Use(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                let mut leaves = Vec::new();
                uses(&item.tree, &[], &mut leaves);
                for (path, name) in leaves {
                    if name == "*" {
                        assert_eq!(
                            path,
                            ["kivact_core", "prelude"],
                            "only the facade prelude glob is allowed"
                        );
                        names.extend(self::names(root, &resolve(root, module, &path)));
                    } else {
                        names.insert(name);
                    }
                }
                None
            }
            _ => None,
        };
        if let Some((syn::Visibility::Public(_), ident)) = declaration {
            names.insert(ident.to_string());
        }
    }
    names
}

fn inventory(
    root: &Path,
    section: &str,
    module: &Module,
    result: &mut BTreeMap<String, BTreeSet<String>>,
) {
    result.insert(section.to_owned(), names(root, module));
    for item in &module.items {
        if let syn::Item::Mod(item) = item
            && matches!(item.vis, syn::Visibility::Public(_))
        {
            inventory(
                root,
                &format!("{section}::{}", item.ident),
                &child(module, item),
                result,
            );
        }
    }
}

fn generate() -> (String, BTreeSet<String>) {
    let root = workspace_root();
    let facade = crate_root(&root, "kivact");
    let mut sections = BTreeMap::new();
    inventory(&root, "kivact", &facade, &mut sections);
    for name in [
        "interaction",
        "ui",
        "primitive",
        "view",
        "style",
        "draw",
        "platform",
        "control",
        "task",
        "motion",
        "i18n",
        "diagnostics",
        "test",
        "gpu_interop",
        "embed",
    ] {
        let module = descend(&root, facade.clone(), &[name.to_owned()]);
        inventory(&root, &format!("kivact::{name}"), &module, &mut sections);
    }
    let mut text = String::from(
        "# Public names\n\nGenerated by `cargo test -p kivact-workspace-policy-tests --test public_names`; bless with `KIVACT_NAMES_BLESS=1`.\nThe counts are guidelines (design §3), not limits.\nNames include feature-gated declarations and exports; methods and trait implementations are excluded.\n",
    );
    let mut qualified = BTreeSet::new();
    for (section, names) in sections {
        text.push_str(&format!("\n## {section} ({})\n\n", names.len()));
        for name in names {
            text.push_str(&format!("- `{name}`\n"));
            qualified.insert(format!("{section}::{name}"));
        }
    }
    (text, qualified)
}

fn golden_names(text: &str) -> BTreeSet<String> {
    let mut section = "";
    let mut names = BTreeSet::new();
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.split_once(" (").expect("section has a count").0;
        } else if let Some(name) = line
            .strip_prefix("- `")
            .and_then(|name| name.strip_suffix('`'))
        {
            names.insert(format!("{section}::{name}"));
        }
    }
    names
}

#[test]
fn public_names_match_the_golden_list() {
    let (generated, generated_names) = generate();
    let golden_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("public-names.golden.md");
    if std::env::var_os("KIVACT_NAMES_BLESS").is_some() {
        std::fs::write(&golden_path, &generated).expect("write the golden file");
        return;
    }
    let golden = std::fs::read_to_string(&golden_path)
        .expect("read public-names.golden.md")
        .replace("\r\n", "\n");
    if golden == generated {
        return;
    }
    let golden_names = golden_names(&golden);
    let mut report = String::new();
    for name in golden_names.difference(&generated_names) {
        report.push_str(&format!("- removed: {name}\n"));
    }
    for name in generated_names.difference(&golden_names) {
        report.push_str(&format!("+ added: {name}\n"));
    }
    panic!(
        "public names differ from public-names.golden.md:\n{report}re-bless with KIVACT_NAMES_BLESS=1 if the change is intended"
    );
}
