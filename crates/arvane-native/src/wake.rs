//! Role: Converts shell wake reasons to atomic native event-loop storage.
use arvane_shell::WakeReasons;

const REASONS: [WakeReasons; 5] = [
    WakeReasons::CLIENT,
    WakeReasons::ASSETS,
    WakeReasons::A11Y_ACTIONS,
    WakeReasons::ENVIRONMENT,
    WakeReasons::GPU_STEP,
];

pub(crate) fn encode(reasons: WakeReasons) -> u8 {
    REASONS
        .into_iter()
        .enumerate()
        .fold(0, |bits, (index, reason)| {
            bits | if reasons.contains(reason) {
                1 << index
            } else {
                0
            }
        })
}

pub(crate) fn decode(bits: u8) -> WakeReasons {
    let mut reasons = WakeReasons::default();
    for (index, reason) in REASONS.into_iter().enumerate() {
        if bits & (1 << index) != 0 {
            reasons.insert(reason);
        }
    }
    reasons
}
