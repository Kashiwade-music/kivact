//! Role: Reads the current system locale.
//!
//! Through the `sys-locale` crate, so no `unsafe` is written here: on
//! Windows `GetUserPreferredUILanguages`, on Unix the environment variables
//! `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` and `LANG`, in that order. Read when
//! a window is created, at `WM_SETTINGCHANGE` (Windows) and whenever a
//! window regains the focus (`runtime.rs`).

/// The system's language and region as a BCP 47 tag (the user's preferred
/// language where the system has one: `"ja-JP"`), or `None` where it cannot
/// tell. The POSIX locales `C` and `POSIX` name no language, so they are
/// `None` too.
pub(crate) fn system_locale() -> Option<String> {
    sys_locale::get_locale().filter(|tag| !matches!(tag.as_str(), "" | "C" | "POSIX"))
}
