//! Role: Store combobox visual dimensions overridden by official kits.
/// Visual measurements for [`crate::combobox`], configured through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct ComboboxTokens {
    pub field_height: f64,
    pub field_padding: f64,
    pub chip_height: f64,
    pub chip_radius: f64,
    pub option_height: f64,
    pub option_padding_x: f64,
    pub option_padding_y: f64,
    pub option_radius: f64,
    pub list_height: f64,
    pub popup_offset: f64,
}
impl Default for ComboboxTokens {
    fn default() -> Self {
        Self {
            field_height: 32.0,
            field_padding: 4.0,
            chip_height: 24.0,
            chip_radius: 4.0,
            option_height: 32.0,
            option_padding_x: 8.0,
            option_padding_y: 6.0,
            option_radius: 4.0,
            list_height: 292.0,
            popup_offset: 6.0,
        }
    }
}
