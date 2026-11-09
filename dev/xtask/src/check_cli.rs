//! Role: Parse local check selection independently of the shared xtask CLI.

pub(crate) const RULES: [&str; 10] = [
    "state-fields",
    "state-fields-strict",
    "effect-writes",
    "api-shape",
    "canonical-path",
    "name-collision",
    "kit-literal-color",
    "os-branch",
    "structural-options",
    "devtools-codegen",
];

#[derive(Default, Debug)]
pub(crate) struct Options {
    pub(crate) only: Option<String>,
    pub(crate) package: Option<String>,
    pub(crate) module: Option<String>,
    pub(crate) report: bool,
}

impl Options {
    pub(crate) fn parse(arguments: Vec<String>) -> Result<(Self, Vec<String>), String> {
        let mut result = Self::default();
        let mut shared = Vec::new();
        let mut args = arguments.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--only" | "--crate" | "--module" => {
                    let value = args
                        .next()
                        .filter(|v| !v.starts_with('-'))
                        .ok_or_else(|| format!("{arg} requires a value"))?;
                    let slot = match arg.as_str() {
                        "--only" => &mut result.only,
                        "--crate" => &mut result.package,
                        _ => &mut result.module,
                    };
                    if slot.replace(value).is_some() {
                        return Err(format!("duplicate {arg}"));
                    }
                }
                "--report" => result.report = true,
                _ => shared.push(arg),
            }
        }
        if let Some(rule) = &result.only
            && !rule.split(',').all(|name| RULES.contains(&name))
        {
            return Err(format!("unknown check: {rule}"));
        }
        if result.module.is_some() && result.package.is_none() {
            return Err("--module requires --crate".into());
        }
        Ok((result, shared))
    }

    pub(crate) fn selected(&self, rule: &str) -> bool {
        self.only
            .as_deref()
            .is_none_or(|only| only.split(',').any(|name| name == rule))
    }
}

#[cfg(test)]
mod tests {
    //! Role: Verify local selectors do not escape into the shared parser.
    use super::Options;
    #[test]
    fn parses_selection_and_rejects_invalid_scopes() {
        let (options, rest) = Options::parse(
            [
                "check",
                "--only",
                "state-fields",
                "--crate",
                "demo",
                "--module",
                "state",
                "--report",
                "--no-write",
            ]
            .map(String::from)
            .to_vec(),
        )
        .unwrap();
        let (selected, _) = Options::parse(vec![
            "--only".into(),
            "api-shape,canonical-path,structural-options".into(),
        ])
        .unwrap();
        assert!(selected.selected("api-shape"));
        assert!(selected.selected("canonical-path"));
        assert!(selected.selected("structural-options"));
        assert!(!selected.selected("state-fields"));
        assert!(Options::parse(vec!["--only".into(), "api-shape,unknown".into()]).is_err());
        assert!(options.report);
        assert_eq!(options.package.as_deref(), Some("demo"));
        assert_eq!(rest, ["check", "--no-write"]);
        assert!(Options::parse(vec!["--module".into(), "state".into()]).is_err());
        assert!(Options::parse(vec!["--only".into(), "unknown".into()]).is_err());
    }
}
