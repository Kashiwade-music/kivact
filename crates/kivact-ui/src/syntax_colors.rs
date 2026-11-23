//! Role: The theme colors of source tokens, one variable per syntax kind.

use kivact_core::style::{Color, ThemeVar, color, rgb};

macro_rules! syntax_color {
    ($var:ident, $name:ident, $init:expr, $doc:literal) => {
        #[doc = $doc]
        pub static $var: ThemeVar<Color> = $init;
        #[doc = $doc]
        pub const $name: Color = Color::var(&$var);
    };
}

syntax_color!(
    KEYWORD_VAR,
    KEYWORD,
    ThemeVar::new("ui.syntax.keyword", rgb(0xcf222e)),
    "Keywords (`fn`, `let`)."
);
syntax_color!(
    TYPE_VAR,
    TYPE,
    ThemeVar::new("ui.syntax.type", rgb(0x953800)),
    "Type names."
);
syntax_color!(
    STRING_VAR,
    STRING,
    ThemeVar::new("ui.syntax.string", rgb(0x0a3069)),
    "String and character literals."
);
syntax_color!(
    COMMENT_VAR,
    COMMENT,
    ThemeVar::new("ui.syntax.comment", rgb(0x6e7781)),
    "Comments."
);
syntax_color!(
    OPERATOR_VAR,
    OPERATOR,
    ThemeVar::alias("ui.syntax.operator", &color::TEXT_VAR),
    "Operators."
);
syntax_color!(
    PUNCTUATION_VAR,
    PUNCTUATION,
    ThemeVar::alias("ui.syntax.punctuation", &color::TEXT_VAR),
    "Punctuation."
);
syntax_color!(
    NUMBER_VAR,
    NUMBER,
    ThemeVar::new("ui.syntax.number", rgb(0x0550ae)),
    "Numeric literals."
);
syntax_color!(
    CONSTANT_VAR,
    CONSTANT,
    ThemeVar::new("ui.syntax.constant", rgb(0x0550ae)),
    "Constants and literals other than numbers."
);
syntax_color!(
    FUNCTION_VAR,
    FUNCTION,
    ThemeVar::new("ui.syntax.function", rgb(0x8250df)),
    "Called or defined functions."
);
syntax_color!(
    VARIABLE_VAR,
    VARIABLE,
    ThemeVar::alias("ui.syntax.variable", &color::TEXT_VAR),
    "Variables."
);
syntax_color!(
    TAG_VAR,
    TAG,
    ThemeVar::new("ui.syntax.tag", rgb(0x116329)),
    "Markup tags."
);
syntax_color!(
    ATTRIBUTE_VAR,
    ATTRIBUTE,
    ThemeVar::new("ui.syntax.attribute", rgb(0x6639ba)),
    "Attributes."
);
syntax_color!(
    PROPERTY_VAR,
    PROPERTY,
    ThemeVar::alias("ui.syntax.property", &color::TEXT_VAR),
    "Properties and fields."
);
syntax_color!(
    MACRO_VAR,
    MACRO,
    ThemeVar::new("ui.syntax.macro", rgb(0x0550ae)),
    "Macro invocations."
);
syntax_color!(
    LIFETIME_VAR,
    LIFETIME,
    ThemeVar::new("ui.syntax.lifetime", rgb(0x116329)),
    "Lifetimes and labels."
);
