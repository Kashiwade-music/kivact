//! Role: The default recipes: the core's look, made only of the theme's tokens.
//!
//! These are the defaults of [`UiRecipes`](super::UiRecipes) for the button,
//! checkbox, switch and text fields; `look_extra` and `look_controls` hold
//! those of the multi-part components. They follow the recipe rules of
//! `super::recipes`: state looks as state styles, the checkbox and switch
//! drawn with local `Style::paint`, and the text fields style only (a native
//! editing leaf has no paint). The default `Link` reads the hover state to
//! underline, the one thing a state style cannot set.
