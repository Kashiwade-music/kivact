//! Role: Validate file payloads and publish accepted selections through one controller.
use super::machine::{DropzoneState, Event, Machine, Validity};
use crate::options_cell::OptionsCell;
use kivact_core::UiError;
use kivact_core::state::{MachineHandle, Value, machine};
use kivact_core::view::HandlerResult;
use kivact_types::platform::FileInfo;
use std::rc::Rc;
type Validator = Rc<dyn Fn(&FileInfo) -> Option<String>>;
type FilesCallback = Rc<dyn Fn(Vec<FileInfo>) -> Result<(), UiError>>;
/// Headless drop target with caller-owned validation and delivery.
#[derive(Clone, Copy)]
pub struct DropzoneApi {
    state: MachineHandle<Machine>,
    validate: OptionsCell<Validator>,
    on_files: OptionsCell<FilesCallback>,
    multiple: bool,
}
/// Create a drop controller. Configure callbacks before attaching it to a view.
pub fn use_dropzone() -> DropzoneApi {
    DropzoneApi {
        state: machine(DropzoneState::Idle, || Machine, |_, ()| {}),
        validate: OptionsCell::new(Rc::new(|_: &FileInfo| None) as Validator),
        on_files: OptionsCell::new(Rc::new(|_| Ok(())) as FilesCallback),
        multiple: false,
    }
}
impl DropzoneApi {
    /// Accept more than one file per operation. Setup only.
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
    /// Supply a pure validator; its first message is shown to the user. Setup only.
    pub fn validate(self, validate: impl Fn(&FileInfo) -> Option<String> + 'static) -> Self {
        self.validate.update(|slot| *slot = Rc::new(validate));
        self
    }
    /// Receive accepted files; a failure becomes the visible failed state. Setup only.
    pub fn on_files<R: HandlerResult>(
        self,
        callback: impl Fn(Vec<FileInfo>) -> R + 'static,
    ) -> Self {
        self.on_files.update(|slot| {
            *slot = Rc::new(move |files| callback(files).into_outcome().map(|_| ()))
        });
        self
    }
    /// Observe drag, picker and error state.
    pub fn state(self) -> Value<DropzoneState> {
        self.state.state()
    }
    fn validity(self, files: &[FileInfo]) -> Validity {
        if !self.multiple && files.len() > 1 {
            return Validity::Invalid("Choose one file.".into());
        }
        files
            .iter()
            .find_map(|file| (self.validate.get())(file))
            .map_or(Validity::Valid, Validity::Invalid)
    }
    /// Validate the current OS drag payload.
    pub fn hover(self, paths: Vec<std::path::PathBuf>) {
        let files: Vec<_> = paths.into_iter().map(FileInfo::from).collect();
        self.state.send(Event::Hover(self.validity(&files)));
    }
    /// End drag feedback.
    pub fn leave(self) {
        self.state.send(Event::Leave);
    }
    /// Begin a native picker, returning false if one is already open.
    pub fn begin_pick(self) -> bool {
        if matches!(self.state.get(), DropzoneState::Picking) {
            return false;
        }
        self.state.send(Event::Pick);
        true
    }
    /// End a canceled picker.
    pub fn cancel_pick(self) {
        self.state.send(Event::Complete(Ok(())));
    }
    /// Display a native picker failure.
    pub fn fail(self, error: UiError) {
        self.state.send(Event::Complete(Err(error)));
    }
    /// Validate and deliver native paths, converting transport data once at this boundary.
    pub fn accept(self, paths: Vec<std::path::PathBuf>) {
        let files: Vec<_> = paths.into_iter().map(FileInfo::from).collect();
        let outcome = match self.validity(&files) {
            Validity::Invalid(message) => Err(UiError::msg(message)),
            Validity::Valid if files.is_empty() => Ok(()),
            Validity::Valid => (self.on_files.get())(files),
        };
        self.state.send(Event::Complete(outcome));
    }
}
