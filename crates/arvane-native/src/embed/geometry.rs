//! Role: Checked logical to physical child sizing.
/// Shared initial/requested sizing rule; the backend supplies its coordinate limit.
#[cfg(any(windows, target_os = "linux"))]
pub(super) fn physical_size(
    size: kivact_types::Size,
    scale: f64,
    max: u32,
) -> Result<(u32, u32), arvane_shell::ShellError> {
    let dimension = |logical: f64| {
        let physical = (logical * scale).round();
        if physical.is_finite() && physical >= 1.0 && physical <= f64::from(max) {
            Ok(physical as u32)
        } else {
            Err(arvane_shell::ShellError::new(
                "scaled dimensions exceed native coordinates",
            ))
        }
    };
    Ok((dimension(size.width)?, dimension(size.height)?))
}
