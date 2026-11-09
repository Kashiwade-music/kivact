//! Role: Load declared Rust modules and preserve source locations for local checks.
//!
//! Following module declarations includes inline modules and #[path] modules in
//! every configuration, without mistaking unreferenced files for crate code.
//! Library and binary targets have distinct identities even within one package;
//! package selectors include declared and autodiscovered binary roots.

use quote::ToTokens;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;

pub(crate) struct Source {
    pub(crate) package: String,
    // xtask-allow(state-fields): option: absent means the source was not reached through a declared Cargo target
    pub(crate) target: Option<String>,
    // xtask-allow(state-fields): independent: a fixed target-kind fact recorded while parsing; any value is valid beside the target name
    pub(crate) library: bool,
    pub(crate) module: String,
    pub(crate) path: PathBuf,
    pub(crate) text: String,
    pub(crate) items: Vec<syn::Item>,
    pub(crate) attrs: Vec<syn::Attribute>,
    start_line: usize,
    end_line: usize,
    excluded: Vec<(usize, usize)>,
    used_allows: RefCell<BTreeSet<usize>>,
}

impl Source {
    pub(crate) fn identity_root(&self) -> String {
        self.target
            .clone()
            .unwrap_or_else(|| self.package.replace('-', "_"))
    }

    pub(crate) fn location(&self, span: proc_macro2::Span) -> String {
        format!(
            "{}:{} ({}::{})",
            self.path.display(),
            span.start().line,
            self.package,
            self.module
        )
    }
    fn marker_before(&self, rule: &str, span: proc_macro2::Span) -> Option<(usize, bool)> {
        let lines: Vec<_> = self.text.lines().collect();
        let mut index = span.start().line.saturating_sub(1);
        while index > 0 {
            index -= 1;
            let line = lines.get(index).copied().unwrap_or("").trim();
            let marker = format!("// xtask-allow({rule}):");
            if let Some(reason) = line.strip_prefix(&marker) {
                return Some((index + 1, !reason.trim().is_empty()));
            }
            if !((line.starts_with("#[") && line.ends_with(']'))
                || line.starts_with("///")
                || line.starts_with("// xtask-allow("))
            {
                break;
            }
        }
        None
    }
    pub(crate) fn allows(&self, rule: &str, span: proc_macro2::Span) -> bool {
        self.allow_line(rule, span).is_some()
    }
    /// The line of the explained marker directly before `span`, without recording its use.
    pub(crate) fn allow_line(&self, rule: &str, span: proc_macro2::Span) -> Option<usize> {
        self.marker_before(rule, span)
            .and_then(|(line, explained)| explained.then_some(line))
    }
    /// The line of the marker directly before `span`, even when its reason is empty.
    pub(crate) fn marker_line(&self, rule: &str, span: proc_macro2::Span) -> Option<usize> {
        self.marker_before(rule, span).map(|(line, _)| line)
    }
    /// Record that the marker on `line` explained a counted field or static.
    pub(crate) fn use_allow(&self, line: usize) {
        self.used_allows.borrow_mut().insert(line);
    }
    pub(crate) fn used_allows(&self) -> BTreeSet<usize> {
        self.used_allows.borrow().clone()
    }
    /// Whether `line` belongs to this module rather than an inline child or another module.
    pub(crate) fn owns_line(&self, line: usize) -> bool {
        self.start_line <= line
            && line <= self.end_line
            && !self
                .excluded
                .iter()
                .any(|(start, end)| *start <= line && line <= *end)
    }
    pub(crate) fn internal_contract(
        &self,
        attrs: &[syn::Attribute],
        span: proc_macro2::Span,
    ) -> bool {
        attrs.iter().any(|attribute| {
            attribute.path().is_ident("doc")
                && attribute.meta.to_token_stream().to_string() == "doc (hidden)"
        }) && self.allows("internal-contract", span)
    }
    pub(crate) fn error(
        &self,
        rule: &str,
        span: proc_macro2::Span,
        detail: &str,
        errors: &mut Vec<String>,
    ) {
        let item_allowed = self.items.iter().any(|item| {
            let parent = item.span();
            parent.start().line <= span.start().line
                && parent.end().line >= span.end().line
                && self.allows(rule, parent)
        });
        if !self.allows(rule, span) && !item_allowed {
            errors.push(format!("{rule}: {}: {detail}", self.location(span)));
        }
    }
    /// Push an error without consulting any marker.
    pub(crate) fn error_at(
        &self,
        rule: &str,
        span: proc_macro2::Span,
        detail: &str,
        errors: &mut Vec<String>,
    ) {
        errors.push(format!("{rule}: {}: {detail}", self.location(span)));
    }
    /// Push an error at a source line that has no parsed span, such as a comment.
    pub(crate) fn error_at_line(
        &self,
        rule: &str,
        line: usize,
        detail: &str,
        errors: &mut Vec<String>,
    ) {
        errors.push(format!(
            "{rule}: {}:{line} ({}::{}): {detail}",
            self.path.display(),
            self.package,
            self.module
        ));
    }
}

pub(crate) fn load(root: &Path) -> Result<Vec<Source>, String> {
    let manifest = read_manifest(&root.join("Cargo.toml"))?;
    let members = manifest
        .get("workspace")
        .and_then(|v| v.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or("workspace.members is required")?;
    let mut sources = Vec::new();
    for member in members {
        let directory = root.join(member.as_str().ok_or("workspace member must be a path")?);
        let manifest = read_manifest(&directory.join("Cargo.toml"))?;
        let package = manifest
            .get("package")
            .and_then(|v| v.get("name"))
            .and_then(toml::Value::as_str)
            .ok_or("package.name is required")?;
        for (name, file, library) in target_roots(&directory, &manifest, package)? {
            let start = sources.len();
            walk_file(package, "", &file, true, &mut BTreeSet::new(), &mut sources)?;
            for source in &mut sources[start..] {
                source.library = library;
                source.target = Some(if library {
                    name.replace('-', "_")
                } else {
                    format!("{}@bin:{}", package.replace('-', "_"), name)
                });
            }
        }
    }
    Ok(sources)
}

/// Discover library and binary roots only; examples, benches and tests have
/// separate user-code contracts and are not additional crate implementations.
fn target_roots(
    directory: &Path,
    manifest: &toml::Value,
    package: &str,
) -> Result<Vec<(String, PathBuf, bool)>, String> {
    let mut roots = Vec::new();
    let lib = manifest.get("lib");
    if lib.is_some()
        || manifest
            .get("package")
            .and_then(|p| p.get("autolib"))
            .and_then(toml::Value::as_bool)
            != Some(false)
    {
        let path = directory.join(
            lib.and_then(|t| t.get("path"))
                .and_then(toml::Value::as_str)
                .unwrap_or("src/lib.rs"),
        );
        if path.is_file() || lib.is_some() {
            let name = lib
                .and_then(|t| t.get("name"))
                .and_then(toml::Value::as_str)
                .unwrap_or(package);
            roots.push((name.to_owned(), path, true));
        }
    }
    let mut binaries = std::collections::BTreeMap::new();
    if let Some(declared) = manifest.get("bin").and_then(toml::Value::as_array) {
        for bin in declared {
            let name = bin
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or(package);
            let path = if let Some(path) = bin.get("path").and_then(toml::Value::as_str) {
                directory.join(path)
            } else if name == package && directory.join("src/main.rs").is_file() {
                directory.join("src/main.rs")
            } else if directory.join(format!("src/bin/{name}.rs")).is_file() {
                directory.join(format!("src/bin/{name}.rs"))
            } else {
                directory.join(format!("src/bin/{name}/main.rs"))
            };
            binaries.insert(name.to_owned(), path);
        }
    }
    let autobins = manifest
        .get("package")
        .and_then(|p| p.get("autobins"))
        .and_then(toml::Value::as_bool)
        != Some(false);
    if autobins {
        if directory.join("src/main.rs").is_file() {
            binaries
                .entry(package.to_owned())
                .or_insert_with(|| directory.join("src/main.rs"));
        }
        let bin_directory = directory.join("src/bin");
        if bin_directory.is_dir() {
            for entry in std::fs::read_dir(&bin_directory)
                .map_err(|e| format!("{}: {e}", bin_directory.display()))?
            {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|v| v == "rs") {
                    let name = path
                        .file_stem()
                        .and_then(|v| v.to_str())
                        .ok_or("binary filename is not UTF-8")?
                        .to_owned();
                    binaries.entry(name).or_insert(path);
                } else if path.is_dir() && path.join("main.rs").is_file() {
                    let name = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| "binary directory is not UTF-8")?;
                    binaries.entry(name).or_insert_with(|| path.join("main.rs"));
                }
            }
        }
    }
    roots.extend(binaries.into_iter().map(|(name, path)| (name, path, false)));
    Ok(roots)
}

fn read_manifest(path: &Path) -> Result<toml::Value, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .parse()
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn walk_file(
    package: &str,
    module: &str,
    file: &Path,
    public: bool,
    seen: &mut BTreeSet<(PathBuf, String)>,
    sources: &mut Vec<Source>,
) -> Result<(), String> {
    if !seen.insert((file.to_path_buf(), module.to_owned())) {
        return Ok(());
    }
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let parsed = syn::parse_file(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    let parent = file.parent().ok_or("source has no parent")?;
    let stem = file
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or("source has no stem")?;
    let directory = if matches!(stem, "lib" | "main" | "mod") {
        parent.to_path_buf()
    } else {
        parent.join(stem)
    };
    let start = sources.len();
    walk_items(
        package,
        module,
        file,
        &text,
        parsed.items,
        &directory,
        parent,
        public,
        1,
        text.lines().count(),
        seen,
        sources,
    )?;
    if let Some(source) = sources[start..].last_mut() {
        source.attrs = parsed.attrs;
    }
    Ok(())
}

fn inline_module_extents(items: &[syn::Item]) -> Vec<(usize, usize)> {
    items
        .iter()
        .filter_map(|item| {
            let syn::Item::Mod(child) = item else {
                return None;
            };
            child.content.as_ref().map(|_| {
                let span = child.span();
                (span.start().line, span.end().line)
            })
        })
        .collect()
}

#[allow(
    clippy::too_many_arguments,
    reason = "the traversal carries source provenance and Rust module lookup context"
)]
fn walk_items(
    package: &str,
    module: &str,
    file: &Path,
    text: &str,
    items: Vec<syn::Item>,
    directory: &Path,
    attribute_directory: &Path,
    public: bool,
    start_line: usize,
    end_line: usize,
    seen: &mut BTreeSet<(PathBuf, String)>,
    sources: &mut Vec<Source>,
) -> Result<(), String> {
    let excluded = inline_module_extents(&items);
    for item in &items {
        let syn::Item::Mod(child) = item else {
            continue;
        };
        let name = if module.is_empty() {
            child.ident.to_string()
        } else {
            format!("{module}::{}", child.ident)
        };
        let child_public = public && matches!(child.vis, syn::Visibility::Public(_));
        if let Some((_, content)) = &child.content {
            let span = child.span();
            walk_items(
                package,
                &name,
                file,
                text,
                content.clone(),
                &directory.join(child.ident.to_string()),
                &directory.join(child.ident.to_string()),
                child_public,
                span.start().line,
                span.end().line,
                seen,
                sources,
            )?;
        } else {
            let explicit = child.attrs.iter().find_map(|attr| {
                if !attr.path().is_ident("path") {
                    return None;
                }
                let syn::Meta::NameValue(value) = &attr.meta else {
                    return None;
                };
                let syn::Expr::Lit(value) = &value.value else {
                    return None;
                };
                let syn::Lit::Str(value) = &value.lit else {
                    return None;
                };
                Some(value.value())
            });
            let path = if let Some(explicit) = explicit {
                let base = attribute_directory;
                base.join(explicit)
            } else {
                let flat = directory.join(format!("{}.rs", child.ident));
                if flat.is_file() {
                    flat
                } else {
                    directory.join(child.ident.to_string()).join("mod.rs")
                }
            };
            walk_file(package, &name, &path, child_public, seen, sources)?;
        }
    }
    sources.push(Source {
        package: package.to_owned(),
        target: None,
        library: true,
        module: module.to_owned(),
        path: file.to_path_buf(),
        text: text.to_owned(),
        items,
        attrs: Vec::new(),
        start_line,
        end_line,
        excluded,
        used_allows: RefCell::new(BTreeSet::new()),
    });
    Ok(())
}

pub(crate) fn type_names(ty: &syn::Type) -> BTreeSet<String> {
    use syn::visit::Visit;
    struct Names(BTreeSet<String>);
    impl<'ast> Visit<'ast> for Names {
        fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
            self.0.insert(segment.ident.to_string());
            syn::visit::visit_path_segment(self, segment);
        }
    }
    let mut names = Names(BTreeSet::new());
    names.visit_type(ty);
    names.0
}

#[cfg(test)]
// An official kit identity keeps kit-only rules active in checker fixtures.
pub(crate) fn fixture(text: &str) -> Source {
    let parsed = syn::parse_file(text).unwrap();
    Source {
        package: "kivact-ui-soft".into(),
        target: None,
        library: true,
        module: "widgets".into(),
        path: "crates/kivact-ui-soft/src/widgets.rs".into(),
        text: text.into(),
        excluded: inline_module_extents(&parsed.items),
        items: parsed.items,
        attrs: parsed.attrs,
        start_line: 1,
        end_line: text.lines().count(),
        used_allows: RefCell::new(BTreeSet::new()),
    }
}

#[cfg(test)]
mod tests {
    //! Role: Verify the two Rust path-attribute bases and diagnostic exceptions.

    #[test]
    fn loads_library_and_binary_targets_without_mixing_type_identities() {
        let root =
            std::env::temp_dir().join(format!("kivact-xtask-targets-{}", std::process::id()));
        let directory = root.join("crates/demo");
        std::fs::create_dir_all(directory.join("src/bin/nested")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = ['crates/demo']",
        )
        .unwrap();
        std::fs::write(
            directory.join("Cargo.toml"),
            "[package]\nname = 'kivact-ui-soft'\n[[bin]]\nname = 'tool'\npath = 'tool.rs'",
        )
        .unwrap();
        std::fs::write(
            directory.join("src/lib.rs"),
            "pub struct Shared { pub x: f32 } pub struct State { a: bool, b: bool }",
        )
        .unwrap();
        std::fs::write(directory.join("src/main.rs"), "pub struct Shared { pub y: f32 } pub struct State { a: bool, b: bool } impl Element for State {}").unwrap();
        for path in ["tool.rs", "src/bin/helper.rs", "src/bin/nested/main.rs"] {
            std::fs::write(directory.join(path), "struct Model { a: bool, b: bool }").unwrap();
        }
        let sources = super::load(&root).unwrap();
        assert_eq!(sources.len(), 5);
        assert_eq!(sources.iter().filter(|s| s.library).count(), 1);
        let identities: std::collections::BTreeSet<_> =
            sources.iter().map(|s| s.identity_root()).collect();
        assert_eq!(identities.len(), 5);
        let surface = crate::public_surface::build(&sources);
        let mut api_errors = Vec::new();
        for source in &sources {
            crate::api_shape::check(source, &surface, &mut api_errors);
        }
        assert_eq!(api_errors.len(), 1);
        let policy = root.join("policy.toml");
        std::fs::write(&policy, "").unwrap();
        let options = crate::check_cli::Options {
            only: Some("state-fields".into()),
            package: Some("kivact-ui-soft".into()),
            ..Default::default()
        };
        assert_eq!(
            crate::local_checks::check(&root, &policy, &options)
                .unwrap()
                .len(),
            4
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn explicit_roots_override_autodiscovery_and_autobins_can_disable_it() {
        let root =
            std::env::temp_dir().join(format!("kivact-xtask-autobins-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(root.join("custom.rs"), "fn main() {}").unwrap();
        let manifest: toml::Value = "[package]\nname = 'demo'\nautobins = false\n[lib]\nname = 'custom_lib'\npath = 'custom_lib.rs'\n[[bin]]\nname = 'demo'\npath = 'custom.rs'".parse().unwrap();
        let roots = super::target_roots(&root, &manifest, "demo").unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(
            roots[0],
            ("custom_lib".into(), root.join("custom_lib.rs"), true)
        );
        assert_eq!(roots[1], ("demo".into(), root.join("custom.rs"), false));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn loads_file_and_inline_path_attributes() {
        let root = std::env::temp_dir().join(format!("kivact-xtask-source-{}", std::process::id()));
        std::fs::create_dir_all(root.join("outer")).unwrap();
        std::fs::write(
            root.join("lib.rs"),
            "mod outer { #[path = \"special.rs\"] mod inner; } mod ordinary;",
        )
        .unwrap();
        std::fs::write(root.join("outer/special.rs"), "pub struct Inline;").unwrap();
        std::fs::write(
            root.join("ordinary.rs"),
            "#[path = \"special.rs\"] mod inner;",
        )
        .unwrap();
        std::fs::write(root.join("special.rs"), "pub struct File;").unwrap();
        let mut sources = Vec::new();
        super::walk_file(
            "demo",
            "",
            &root.join("lib.rs"),
            true,
            &mut Default::default(),
            &mut sources,
        )
        .unwrap();
        assert!(
            sources
                .iter()
                .any(|s| s.module == "outer::inner" && s.path.ends_with("outer/special.rs"))
        );
        assert!(
            sources
                .iter()
                .any(|s| s.module == "ordinary::inner" && s.path.ends_with("special.rs"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn field_and_item_markers_require_nonempty_reasons() {
        use syn::spanned::Spanned;
        let source = super::fixture(
            "// xtask-allow(api-shape): wire format\npub struct Wire { pub x: f32 }\n// xtask-allow(api-shape):\npub struct Unexplained { pub x: f32 }",
        );
        let mut errors = Vec::new();
        for item in &source.items {
            if let syn::Item::Struct(item) = item {
                for field in &item.fields {
                    source.error("api-shape", field.span(), "f32", &mut errors);
                }
            }
        }
        assert_eq!(errors.len(), 1);
    }
}
