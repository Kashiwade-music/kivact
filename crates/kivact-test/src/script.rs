//! Role: Public write-only script for virtual platform prerequisites and input.
#[derive(Clone)]
pub struct Script {
    pub(crate) platform: arvane_virtual::Script,
}
impl Script {
    pub fn respond(
        &self,
        f: impl FnMut(&kivact_types::PlatformRequest) -> Option<kivact_types::PlatformResult> + 'static,
    ) {
        self.platform.respond(f);
    }
    pub fn raw(&self, event: crate::raw::RawInput) {
        self.platform.raw(event);
    }
    pub fn mouse(&self, phase: kivact_types::PointerPhase, position: Option<kivact_types::Point>) {
        self.platform.mouse(phase, position);
    }
    pub fn contact(&self, contact: kivact_types::PointerSample) {
        self.platform.contact(contact);
    }
    pub fn focus(&self, focused: bool) {
        self.platform.focus(focused);
    }
    pub fn accessibility_action(&self, node: u64, request: kivact_types::A11yRequest) {
        self.platform.accessibility_action(node, request);
    }
    pub fn set_now(&self, ns: u64) {
        self.platform.set_now(ns);
    }
    pub fn resize(&self, size: kivact_types::Size) {
        self.platform.resize(size);
    }
    pub fn set_scale(&self, scale: f64) {
        self.platform.set_scale(scale);
    }
    pub fn set_frame_period(&self, ns: u64) {
        self.platform.set_frame_period(ns);
    }
    pub fn set_reduced_motion(&self, value: bool) {
        self.platform.set_reduced_motion(value);
    }
    pub fn set_dark(&self, value: bool) {
        self.platform.set_dark(value);
    }
    pub fn set_locale(&self, locale: Option<&str>) {
        self.platform.set_locale(locale);
    }
    pub fn set_safe_area(&self, insets: kivact_types::Insets) {
        self.platform.set_safe_area(insets);
    }
    pub fn create_surface(&self) {
        self.platform.create_surface();
    }
    pub fn destroy_surface(&self) {
        self.platform.destroy_surface();
    }
    pub fn set_occluded(&self, value: bool) {
        self.platform.set_occluded(value);
    }
    pub fn fail_presents(&self, count: u32) {
        self.platform.fail_presents(count);
    }
    pub fn set_clipboard(&self, text: &str) {
        self.platform.set_clipboard(text);
    }
    pub fn push_file_dialog(&self, path: Option<std::path::PathBuf>) {
        self.platform.push_file_dialog(path);
    }
    pub fn push_files_dialog(&self, paths: Option<Vec<std::path::PathBuf>>) {
        self.platform.push_files_dialog(paths);
    }
    pub fn fail_next_effect(&self, reason: &str) {
        self.platform.fail_next_effect(reason);
    }
    pub fn fail_next_effect_unsupported(&self, reason: &str) {
        self.platform.fail_next_effect_unsupported(reason);
    }
}
