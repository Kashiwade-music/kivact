//! Role: The page, example and preview types of a catalog, with their builders.

use kivact::view::{IntoView, View};

/// A documentation page of one component.
pub struct Page {
    /// The kebab-case slug: `catalog-nav-<slug>` and `example-<slug>-<id>-*` test ids.
    pub(crate) slug: &'static str,
    /// The English component name: the list entry and the page title.
    pub(crate) name: &'static str,
    /// The heading the list shows above the page's group of entries.
    pub(crate) category: &'static str,
    /// An English description under the title; empty for none.
    pub(crate) description: &'static str,
    /// The imports shown under "Usage"; empty hides that section.
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
pub enum PreviewAlign {
    /// Centered: the default.
    Center,
    /// At the top.
    Start,
    /// At the bottom.
    End,
}

/// One demo of a page: a title, an English explanation and the view to show.
pub struct Example {
    /// The id of the `// example: <id>` marker; also part of the test ids.
    pub(crate) id: &'static str,
    /// An English title.
    pub(crate) title: &'static str,
    /// An English explanation under the title.
    pub(crate) description: &'static str,
    /// Build the demo.
    view: Box<dyn Fn() -> View>,
}

impl Page {
    /// A page with the slug, the name, the category heading and the page
    /// file's source; no description or usage, centered previews, no examples.
    pub fn new(
        slug: &'static str,
        name: &'static str,
        category: &'static str,
        source: &'static str,
    ) -> Page {
        Page {
            slug,
            name,
            category,
            description: "",
            usage: "",
            source,
            align: PreviewAlign::Center,
            examples: Vec::new(),
        }
    }

    /// The page with `text` under its title.
    #[must_use]
    pub fn description(mut self, text: &'static str) -> Page {
        self.description = text;
        self
    }

    /// The page with the imports `code` shown under "Usage".
    #[must_use]
    pub fn usage(mut self, code: &'static str) -> Page {
        self.usage = code;
        self
    }

    /// The page with its previews aligned at `align`.
    #[must_use]
    pub fn align(mut self, align: PreviewAlign) -> Page {
        self.align = align;
        self
    }

    /// The page with `example` added after the examples it has.
    #[must_use]
    pub fn example(mut self, example: Example) -> Page {
        self.examples.push(example);
        self
    }

    /// The dedented source between `// example: <id>` and `// end example`.
    pub fn snippet(&self, id: &str) -> String {
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

impl Example {
    /// An example whose demo is `view`, any function returning a view (the
    /// type is erased here, so each demo is a plain `fn default() -> impl IntoView`).
    pub fn new<V: IntoView>(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        view: fn() -> V,
    ) -> Example {
        Example {
            id,
            title,
            description,
            view: Box::new(move || view().into_view()),
        }
    }

    /// Build the demo.
    pub fn view(&self) -> View {
        (self.view)()
    }
}
