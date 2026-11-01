//! Role: Page and example types shared by the catalog shell and its page files.

use kivact::view::{IntoView, View};

/// A documentation page of one component.
pub(crate) struct Page {
    /// The kebab-case slug: `catalog-nav-<slug>` and `example-<slug>-<id>-*` test ids.
    pub(crate) slug: &'static str,
    /// The English component name: the list entry and the page title.
    pub(crate) name: &'static str,
    /// A Japanese description under the title.
    pub(crate) description: &'static str,
    /// The imports shown under "使い方".
    pub(crate) usage: &'static str,
    /// The page file's own source: `include_str!("<slug>.rs")`.
    pub(crate) source: &'static str,
    /// Where the previews hold their demo along the height of the box.
    pub(crate) align: PreviewAlign,
    /// The examples in page order.
    pub(crate) examples: Vec<Example>,
}

/// The vertical place of a demo in its preview box (shadcn's
/// `ComponentPreview align`). A demo that grows when it is used (an
/// accordion, a collapsible, a command list) starts at the top, so its first
/// row stays put while the rest opens below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreviewAlign {
    /// Centered: the default.
    Center,
    /// At the top.
    Start,
    /// At the bottom.
    End,
}

/// One demo of a page: a title, a Japanese explanation and the view to show.
pub(crate) struct Example {
    /// The id of the `// example: <id>` marker; also part of the test ids.
    pub(crate) id: &'static str,
    /// An English title.
    pub(crate) title: &'static str,
    /// A Japanese explanation under the title.
    pub(crate) description: &'static str,
    /// Build the demo.
    view: Box<dyn Fn() -> View + Send + Sync>,
}

impl Example {
    /// An example whose demo is `view`, any function returning a view (the
    /// type is erased here, so each demo is a plain `fn default() -> impl IntoView`).
    pub(crate) fn new<V: IntoView>(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        view: fn() -> V,
    ) -> Self {
        Self {
            id,
            title,
            description,
            view: Box::new(move || view().into_view()),
        }
    }

    /// Build the demo.
    pub(crate) fn view(&self) -> View {
        (self.view)()
    }
}

impl Page {
    /// The dedented source between `// example: <id>` and `// end example`.
    pub(crate) fn snippet(&self, id: &str) -> String {
        let begin = format!("// example: {id}");
        let mut lines: Vec<&str> = Vec::new();
        let mut inside = false;
        for line in self.source.lines() {
            let trimmed = line.trim();
            if inside {
                if trimmed == "// end example" {
                    break;
                }
                lines.push(line);
            } else if trimmed == begin {
                inside = true;
            }
        }
        let indent = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        lines
            .iter()
            .map(|line| line.get(indent..).unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
