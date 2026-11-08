//! Role: Public script for virtual platform prerequisites, input and saved state.
#[derive(Clone)]
pub struct Script {
    pub(crate) platform: arvane_virtual::Script,
    pub(crate) window: kivact_types::WindowId,
}
impl Script {
    /// Saved bytes to pass to `RenderOptions::restored_state` on the next launch.
    pub fn saved_state(&self) -> std::collections::BTreeMap<String, Vec<u8>> {
        self.platform.saved_state()
    }
    /// Request a lifecycle save notification.
    pub fn save_state(&self) {
        self.platform.save_state();
    }
    /// Report software keyboard occlusion.
    pub fn set_keyboard_inset(&self, insets: kivact_types::Insets) {
        self.platform.window(self.window).set_keyboard_inset(insets);
    }
    pub fn respond(
        &self,
        f: impl FnMut(&kivact_types::PlatformRequest) -> Option<kivact_types::PlatformResult> + 'static,
    ) {
        self.platform.respond(f);
    }
    pub fn raw(&self, event: crate::raw::RawInput) {
        self.platform.window(self.window).raw(event);
    }
    pub fn mouse(&self, phase: kivact_types::PointerPhase, position: Option<kivact_types::Point>) {
        self.platform.window(self.window).mouse(phase, position);
    }
    pub fn contact(&self, contact: kivact_types::PointerSample) {
        self.platform.window(self.window).contact(contact);
    }
    pub fn focus(&self, focused: bool) {
        self.platform.window(self.window).focus(focused);
    }
    pub fn accessibility_action(&self, node: u64, request: kivact_types::A11yRequest) {
        self.platform
            .window(self.window)
            .accessibility_action(node, request);
    }
    pub fn set_now(&self, ns: u64) {
        self.platform.set_now(ns);
    }
    pub fn resize(&self, size: kivact_types::Size) {
        self.platform.window(self.window).resize(size);
    }
    pub fn set_scale(&self, scale: f64) {
        self.platform.window(self.window).set_scale(scale);
    }
    pub fn set_frame_period(&self, ns: u64) {
        self.platform.window(self.window).set_frame_period(ns);
    }
    pub fn set_reduced_motion(&self, value: bool) {
        self.platform.window(self.window).set_reduced_motion(value);
    }
    pub fn set_dark(&self, value: bool) {
        self.platform.window(self.window).set_dark(value);
    }
    pub fn set_locale(&self, locale: Option<&str>) {
        self.platform.window(self.window).set_locale(locale);
    }
    pub fn set_safe_area(&self, insets: kivact_types::Insets) {
        self.platform.window(self.window).set_safe_area(insets);
    }
    pub fn create_surface(&self) {
        self.platform.window(self.window).create_surface();
    }
    pub fn destroy_surface(&self) {
        self.platform.window(self.window).destroy_surface();
    }
    pub fn set_occluded(&self, value: bool) {
        self.platform.window(self.window).set_occluded(value);
    }
    pub fn fail_presents(&self, count: u32) {
        self.platform.window(self.window).fail_presents(count);
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
