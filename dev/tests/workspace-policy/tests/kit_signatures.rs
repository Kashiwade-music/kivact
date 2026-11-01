//! Role: Keep documented common kit constructor inputs identical across kits.
mod kit_signatures {
    //! Role: Read public constructors and normalize their declared Rust inputs.
    pub(super) mod normalize;
    pub(super) mod source;
}

use kit_signatures::{normalize, source};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const NAMES: [&str; 7] = [
    "dropzone",
    "skeleton",
    "pagination",
    "stepper",
    "empty_state",
    "breadcrumb",
    "alert",
];
const KITS: [&str; 4] = ["basic", "game", "pop", "creative"];
/// Spellings no kit exports: `empty_state` and `breadcrumb` are the names
/// (see `docs/for-component-author/kit-naming.md`).
const FORBIDDEN_EXPORTS: [&str; 2] = ["empty", "breadcrumbs"];

struct Documented {
    name: String,
    kits: Vec<String>,
    signature: syn::Signature,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn unquote(text: &str) -> &str {
    text.trim().trim_matches('`')
}

fn documented(root: &Path) -> Vec<Documented> {
    let source = std::fs::read_to_string(root.join("docs/for-component-author/kit-naming.md"))
        .expect("read kit naming policy");
    let section = source
        .split_once("## 共通の構成関数の署名")
        .expect("kit naming has the common constructor section")
        .1;
    let mut rows = Vec::new();
    for line in section.lines().take_while(|line| !line.starts_with("## ")) {
        if !line.starts_with('|') {
            continue;
        }
        let cells = line
            .trim_matches('|')
            .split('|')
            .map(unquote)
            .collect::<Vec<_>>();
        assert_eq!(
            cells.len(),
            3,
            "common constructor table has three columns: {line}"
        );
        if cells == ["名前", "kit", "入力の署名"]
            || cells
                .iter()
                .all(|cell| cell.chars().all(|c| c == '-' || c == ':'))
        {
            continue;
        }
        let name = cells[0].to_owned();
        let kits = if cells[1].contains("標準のみ") {
            assert_eq!(name, "field", "only field is documented as standard-only");
            vec!["ui".into()]
        } else {
            cells[1]
                .split(',')
                .map(|kit| kit.trim().to_owned())
                .collect()
        };
        let signature =
            syn::parse_str(&format!("fn documented{}", cells[2])).unwrap_or_else(|error| {
                panic!("{name}: invalid documented inputs {}: {error}", cells[2])
            });
        rows.push(Documented {
            name,
            kits,
            signature,
        });
    }
    assert!(!rows.is_empty(), "common constructor table cannot be empty");
    rows
}

#[test]
fn common_kit_constructor_inputs_match_the_documented_signatures() {
    let root = workspace_root();
    let rows = documented(&root);
    let names = rows
        .iter()
        .map(|row| row.name.as_str())
        .collect::<BTreeSet<_>>();
    let expected = NAMES.into_iter().chain(["field"]).collect::<BTreeSet<_>>();
    assert_eq!(
        names, expected,
        "the table must cover exactly the seven common names and standard-only field"
    );
    let mut documented = BTreeSet::new();
    let mut common = BTreeMap::new();
    for row in rows {
        assert!(!row.kits.is_empty(), "{}: missing kit list", row.name);
        let expected = normalize::signature(&row.signature, None);
        for kit in row.kits {
            assert!(
                KITS.contains(&kit.as_str()) || kit == "ui",
                "{}: unknown kit {kit}",
                row.name
            );
            assert_eq!(
                kit == "ui",
                row.name == "field",
                "field is standard-only and common constructors belong to kits"
            );
            assert!(
                documented.insert((kit.clone(), row.name.clone())),
                "{kit}/{}: duplicate table entry",
                row.name
            );
            let crate_name = if kit == "ui" {
                "kivact_ui".to_owned()
            } else {
                format!("kivact_ui_{kit}")
            };
            let (signature, module) = source::constructor(&root, &crate_name, &row.name).unwrap_or_else(|| panic!("{kit}/{}: documented public constructor is not explicitly exported; expected {expected}; actual <missing>", row.name));
            // Field is an ownership assertion, not a kit signature comparison.
            // Its standard builder uses named generics, outside this policy's scope.
            if kit == "ui" {
                continue;
            }
            let actual = normalize::signature(&signature, Some(&module));
            assert_eq!(
                actual, expected,
                "{kit}/{}: expected documented normalized inputs {expected}; actual {actual}",
                row.name
            );
            if let Some((previous_kit, previous)) =
                common.insert(row.name.clone(), (kit.clone(), actual.clone()))
            {
                assert_eq!(
                    actual, previous,
                    "{kit}/{}: expected {previous_kit}'s normalized inputs {previous}; actual {actual}",
                    row.name
                );
            }
        }
    }
    for kit in KITS {
        let crate_name = format!("kivact_ui_{kit}");
        for name in NAMES.into_iter().chain(["field"]) {
            let actual = source::constructor(&root, &crate_name, name);
            let listed = documented.contains(&(kit.to_owned(), name.to_owned()));
            assert_eq!(
                actual.is_some(),
                listed,
                "{kit}/{name}: expected table coverage {listed}; actual public constructor {}",
                actual.as_ref().map_or_else(
                    || "<missing>".into(),
                    |(sig, module)| normalize::signature(sig, Some(module))
                )
            );
        }
    }
}

fn item_name(item: &syn::Item) -> Option<&syn::Ident> {
    match item {
        syn::Item::Fn(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.sig.ident)
        }
        syn::Item::Struct(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.ident)
        }
        syn::Item::Enum(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.ident)
        }
        syn::Item::Type(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.ident)
        }
        syn::Item::Const(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.ident)
        }
        syn::Item::Static(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.ident)
        }
        syn::Item::Trait(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(&item.ident)
        }
        _ => None,
    }
}

/// The signature comparison follows named exports only, so a kit root that
/// exposes a glob or a public module would hide constructors from it, and
/// the spellings the naming policy unified must not come back.
#[test]
fn kit_roots_name_each_export_and_reject_unified_spellings() {
    let root = workspace_root();
    for kit in KITS {
        let module = source::crate_root(&root, &format!("kivact_ui_{kit}"));
        for item in &module.items {
            let mut exposed = Vec::new();
            match item {
                syn::Item::Mod(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                    panic!(
                        "{kit}: `pub mod {}`: kit roots must name each export so their signatures can be compared",
                        item.ident
                    );
                }
                syn::Item::Use(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                    let mut leaves = Vec::new();
                    source::uses(&item.tree, &[], &mut leaves);
                    for (path, name) in leaves {
                        assert!(
                            name != "*",
                            "{kit}: `pub use {}::*`: kit roots must name each export so their signatures can be compared",
                            path.join("::")
                        );
                        exposed.push(name);
                    }
                }
                item => exposed.extend(item_name(item).map(ToString::to_string)),
            }
            for name in exposed {
                assert!(
                    !FORBIDDEN_EXPORTS.contains(&name.as_str()),
                    "{kit}: exports `{name}`; use the names in docs/for-component-author/kit-naming.md"
                );
            }
        }
    }
}
