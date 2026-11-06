//! Role: Uses the desktop clipboard and dialog services on the child UI thread.

use super::conn::Native;
use arvane_shell::{FileDialog, ShellError};
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct Services(crate::services::Services);

impl Services {
    pub(super) fn copy(&mut self, text: &str) -> Result<(), ShellError> {
        crate::module::retain_module();
        self.0.copy_text(text)
    }
    pub(super) fn read(&mut self) -> Result<String, ShellError> {
        crate::module::retain_module();
        self.0.read_text()
    }
    pub(super) fn pick_file(
        native: &Native,
        request: &FileDialog,
    ) -> Result<Option<PathBuf>, ShellError> {
        crate::module::retain_module();
        crate::services::Services::pick_file(native, request)
    }
    pub(super) fn pick_files(
        native: &Native,
        request: &FileDialog,
    ) -> Result<Option<Vec<PathBuf>>, ShellError> {
        crate::module::retain_module();
        crate::services::Services::pick_files(native, request)
    }
    pub(super) fn pick_folder(
        native: &Native,
        request: &FileDialog,
    ) -> Result<Option<PathBuf>, ShellError> {
        crate::module::retain_module();
        crate::services::Services::pick_folder(native, request)
    }
    pub(super) fn save_file(
        native: &Native,
        request: &FileDialog,
    ) -> Result<Option<PathBuf>, ShellError> {
        crate::module::retain_module();
        crate::services::Services::save_file(native, request)
    }
}
