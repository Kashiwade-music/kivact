//! Role: Window-owned clipboard and native file dialog services.
//! Errors stay at the platform boundary.

use std::path::PathBuf;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use arvane_shell::{ShellError, ShellFileDialog};

#[derive(Default)]
pub(crate) struct Services {
    pub(crate) clipboard: Option<arboard::Clipboard>,
}

impl Services {
    fn clipboard(&mut self) -> Result<&mut arboard::Clipboard, ShellError> {
        if self.clipboard.is_none() {
            self.clipboard = Some(arboard::Clipboard::new().map_err(platform_error)?);
        }
        self.clipboard
            .as_mut()
            .ok_or_else(|| ShellError::new("clipboard initialization did not produce a handle"))
    }

    pub(crate) fn copy_text(&mut self, text: &str) -> Result<(), ShellError> {
        self.clipboard()?.set_text(text).map_err(platform_error)
    }

    pub(crate) fn read_text(&mut self) -> Result<String, ShellError> {
        self.clipboard()?.get_text().map_err(platform_error)
    }

    pub(crate) fn pick_file(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        request: &ShellFileDialog,
    ) -> Result<Option<PathBuf>, ShellError> {
        Ok(dialog(window, request)?.pick_file())
    }

    pub(crate) fn pick_files(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        request: &ShellFileDialog,
    ) -> Result<Option<Vec<PathBuf>>, ShellError> {
        Ok(dialog(window, request)?.pick_files())
    }

    /// The file name and filters of `request` do not apply to a folder;
    /// they are still validated like every dialog's.
    pub(crate) fn pick_folder(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        request: &ShellFileDialog,
    ) -> Result<Option<PathBuf>, ShellError> {
        Ok(dialog(window, request)?.pick_folder())
    }

    pub(crate) fn save_file(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        request: &ShellFileDialog,
    ) -> Result<Option<PathBuf>, ShellError> {
        Ok(dialog(window, request)?.save_file())
    }
}

fn platform_error(error: arboard::Error) -> ShellError {
    ShellError::new(format!("clipboard: {error}"))
}

fn dialog(
    window: &(impl HasWindowHandle + HasDisplayHandle),
    request: &ShellFileDialog,
) -> Result<rfd::FileDialog, ShellError> {
    validate(request)?;
    let mut dialog = rfd::FileDialog::new()
        .set_parent(window)
        .set_title(&request.title);
    if let Some(directory) = &request.directory {
        dialog = dialog.set_directory(directory);
    }
    if let Some(file_name) = &request.file_name {
        dialog = dialog.set_file_name(file_name);
    }
    for (label, extensions) in &request.filters {
        dialog = dialog.add_filter(label, extensions);
    }
    Ok(dialog)
}

fn validate(request: &ShellFileDialog) -> Result<(), ShellError> {
    if request.title.contains('\0')
        || request
            .file_name
            .as_ref()
            .is_some_and(|name| name.contains('\0'))
        || request.filters.iter().any(|(label, extensions)| {
            label.contains('\0')
                || extensions.is_empty()
                || extensions.iter().any(|extension| {
                    extension.is_empty() || extension.contains(['\0', '.', '/', '\\'])
                })
        })
    {
        return Err(ShellError::new("invalid native file dialog options"));
    }
    Ok(())
}
