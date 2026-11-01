//! Role: Doctest harness for the Markdown guides in `docs/`.
//!
//! Each guide is included as the documentation of an empty module, so
//! `cargo test -p kivact-docs --doc` compiles (and, unless marked `no_run`,
//! runs) every ```` ```rust ```` block. Blocks that open a window are
//! `no_run`; blocks that are not Rust use ```` ```text ````.
//!
//! A block that shows an app together with its `#[cfg(test)] mod tests` is
//! `test_harness`: rustdoc builds it with `--test`, so the tests compile and
//! run while `main` (which opens a window) is never called. Under `no_run`
//! the `cfg(test)` module would not even be compiled; a unit test of this
//! crate rejects such blocks.

macro_rules! guide {
    ($($name:ident => $path:literal),* $(,)?) => {
        $(
            #[doc = include_str!($path)]
            pub mod $name {}
        )*
    };
}

/// The repository README.
#[doc = include_str!("../../../../README.md")]
pub mod readme {
    //! Role: The repository README as a doctest.
}

/// `docs/for-kivact-user/getting-started`.
pub mod user {
    //! Role: Getting started guides for Kivact users as doctests.
    //!
    guide! {
        overview => "../../../../docs/for-kivact-user/README.md",
        readme => "../../../../docs/for-kivact-user/getting-started/README.md",
        quickstart => "../../../../docs/for-kivact-user/getting-started/quickstart.md",
        components_layout => "../../../../docs/for-kivact-user/getting-started/components-layout.md",
        react_solid => "../../../../docs/for-kivact-user/getting-started/react-solid.md",
        state => "../../../../docs/for-kivact-user/getting-started/state.md",
        input_events => "../../../../docs/for-kivact-user/getting-started/input-events.md",
        conditional_lists => "../../../../docs/for-kivact-user/getting-started/conditional-lists.md",
        styling_accessibility => "../../../../docs/for-kivact-user/getting-started/styling-accessibility.md",
        async_tasks => "../../../../docs/for-kivact-user/getting-started/async.md",
        testing => "../../../../docs/for-kivact-user/getting-started/testing.md",
    }
}

/// The design document and the coding rules.
pub mod developer {
    //! Role: Design document and coding rules as doctests.
    //!
    guide! {
        design => "../../../../docs/for-kivact-developer/design.md",
        coding_rules => "../../../../docs/for-kivact-developer/coding-rules.md",
    }
}

#[cfg(test)]
mod tests;

/// Official kit tutorials and task guides.
pub mod official_ui {
    //! Role: Official UI kit tutorials and guides as doctests.
    //!
    guide! {
        basic_tutorial => "../../../../docs/for-kivact-ui-user/getting-started/basic/README.md",
        guide_0 => "../../../../docs/for-kivact-ui-user/getting-started/pop/README.md",
        guide_1 => "../../../../docs/for-kivact-ui-user/getting-started/creative/README.md",
        guide_2 => "../../../../docs/for-kivact-ui-user/getting-started/game/README.md",
        guide_7 => "../../../../docs/for-kivact-ui-user/README.md",
        how_to_readme => "../../../../docs/for-kivact-ui-user/how-to-guide/README.md",
        basic_tasks => "../../../../docs/for-kivact-ui-user/how-to-guide/basic.md",
        pop_tasks => "../../../../docs/for-kivact-ui-user/how-to-guide/pop.md",
        creative_tasks => "../../../../docs/for-kivact-ui-user/how-to-guide/creative.md",
        game_tasks => "../../../../docs/for-kivact-ui-user/how-to-guide/game.md",
    }
}

/// Task guides for application authors.
pub mod user_how_to {
    //! Role: Application task guides as doctests.
    //!
    guide! {
        readme => "../../../../docs/for-kivact-user/how-to-guide/README.md",
        embed => "../../../../docs/for-kivact-user/how-to-guide/embed.md",
        forms => "../../../../docs/for-kivact-user/how-to-guide/forms.md",
        lists_tables => "../../../../docs/for-kivact-user/how-to-guide/lists-tables.md",
        overlays => "../../../../docs/for-kivact-user/how-to-guide/overlays.md",
        canvas => "../../../../docs/for-kivact-user/how-to-guide/canvas.md",
        animation => "../../../../docs/for-kivact-user/how-to-guide/animation.md",
        platform => "../../../../docs/for-kivact-user/how-to-guide/platform.md",
        i18n => "../../../../docs/for-kivact-user/how-to-guide/i18n.md",
        theming => "../../../../docs/for-kivact-user/how-to-guide/theming.md",
        diagnostics => "../../../../docs/for-kivact-user/how-to-guide/diagnostics.md",
    }
}

/// Public interaction and primitive contracts for component authors.
pub mod component_author {
    //! Role: Component author guides as doctests.
    //!
    guide! {
        readme => "../../../../docs/for-component-author/README.md",
        interaction => "../../../../docs/for-component-author/interaction.md",
        primitive => "../../../../docs/for-component-author/primitive.md",
        kit_naming => "../../../../docs/for-component-author/kit-naming.md",
    }
}
