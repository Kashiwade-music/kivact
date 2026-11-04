//! Role: Substitute named arguments in message templates.

/// `template` with each `{name}` replaced by the value `args` holds for
/// `name` (what `message_with` does on each change). For code that builds
/// a text where no setup is available, such as a drawing closure: read the
/// template once in setup with `message` (a drawing closure may read the
/// returned value, and runs again when it changes), then fill it where the
/// values are known. A `{name}` that `args` does not name stays as written.
///
/// ```
/// use kivact::i18n::interpolate;
///
/// assert_eq!(
///     interpolate("Link {from} to {to}", &[("from", "A"), ("to", "B")]),
///     "Link A to B"
/// );
/// assert_eq!(interpolate("{x} and {y}", &[("x", "1")]), "1 and {y}");
/// ```
pub fn interpolate(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let tail = &rest[open + 1..];
        match tail.find('}') {
            Some(close) => {
                let name = &tail[..close];
                match args.iter().find(|(key, _)| *key == name) {
                    Some((_, value)) => out.push_str(value),
                    None => out.push_str(&rest[open..open + close + 2]),
                }
                rest = &tail[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}
