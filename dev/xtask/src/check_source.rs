//! Role: Load declared Rust modules and preserve source locations for local checks.
//!
//! Following module declarations includes inline modules and #[path] modules in
//! every configuration, without mistaking unreferenced files for crate code.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;

pub(crate) struct Source {
    pub(crate) package: String,
    pub(crate) module: String,
    pub(crate) path: PathBuf,
    pub(crate) text: String,
    pub(crate) items: Vec<syn::Item>,
    pub(crate) attrs: Vec<syn::Attribute>,
}

impl Source {
    pub(crate) fn location(&self, span: proc_macro2::Span) -> String {
        format!(
            "{}:{} ({}::{})",
            self.path.display(),
            span.start().line,
            self.package,
            self.module
        )
    }
    pub(crate) fn allows(&self, rule: &str, span: proc_macro2::Span) -> bool {
        let lines: Vec<_> = self.text.lines().collect();
        let mut index = span.start().line.saturating_sub(1);
        while index > 0 {
            index -= 1;
            let line = lines.get(index).copied().unwrap_or("").trim();
            let marker = format!("// xtask-allow({rule}):");
            if let Some(reason) = line.strip_prefix(&marker) {
                return !reason.trim().is_empty();
            }
            if !(line.starts_with("#[")
                || line.starts_with("///")
                || line.starts_with("// xtask-allow("))
            {
                break;
            }
        }
        false
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
        let lib = manifest
            .get("lib")
            .and_then(|v| v.get("path"))
            .and_then(toml::Value::as_str)
            .unwrap_or("src/lib.rs");
        let file = directory.join(lib);
        if file.is_file() {
            walk_file(package, "", &file, true, &mut BTreeSet::new(), &mut sources)?;
        } else if directory.join("src/main.rs").is_file() {
            walk_file(
                package,
                "",
                &directory.join("src/main.rs"),
                true,
                &mut BTreeSet::new(),
                &mut sources,
            )?;
        }
    }
    Ok(sources)
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
        seen,
        sources,
    )?;
    if let Some(source) = sources[start..].last_mut() {
        source.attrs = parsed.attrs;
    }
    Ok(())
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
    seen: &mut BTreeSet<(PathBuf, String)>,
    sources: &mut Vec<Source>,
) -> Result<(), String> {
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
            walk_items(
                package,
                &name,
                file,
                text,
                content.clone(),
                &directory.join(child.ident.to_string()),
                &directory.join(child.ident.to_string()),
                child_public,
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
        module: module.to_owned(),
        path: file.to_path_buf(),
        text: text.to_owned(),
        items,
        attrs: Vec::new(),
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
pub(crate) fn fixture(text: &str) -> Source {
    Source {
        package: "kivact-ui-demo".into(),
        module: "widgets".into(),
        path: "crates/kivact-ui-demo/src/widgets.rs".into(),
        text: text.into(),
        items: syn::parse_file(text).unwrap().items,
        attrs: syn::parse_file(text).unwrap().attrs,
    }
}

#[cfg(test)]
mod tests {
    //! Role: Verify the two Rust path-attribute bases and diagnostic exceptions.
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
