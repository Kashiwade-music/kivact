//! Role: The Direct3D 12 helpers the pool and the read back share: barrier, wait.

#![allow(
    unsafe_code,
    reason = "Direct3D 12 and Win32 calls, which windows marks unsafe"
)]

use crate::common::{Error, api};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::ID3D12Fence;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::PCWSTR;

/// The pool textures' format (premultiplied alpha).
///
/// The values are sRGB-encoded: the window composites them as it does
/// `Pixels`. An engine that computes in linear light encodes to sRGB
/// last; the texture is not sampled through an sRGB view.
pub const FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;

/// A transition barrier borrowing `resource` (no reference is taken).
pub(crate) use crate::d3d12_contract::transition;

/// Wait at most `timeout` on the CPU for `fence` to reach `value`.
pub(crate) fn wait(
    fence: &ID3D12Fence,
    value: u64,
    timeout: Duration,
    what: &'static str,
) -> Result<(), Error> {
    // SAFETY: a live fence.
    if unsafe { fence.GetCompletedValue() } >= value {
        return Ok(());
    }
    // SAFETY: an unnamed auto-reset event.
    let event =
        unsafe { CreateEventW(None, false, false, PCWSTR::null()) }.map_err(api("CreateEventW"))?;
    // SAFETY: the new event's handle, owned here alone (closed on return).
    let event = unsafe { OwnedHandle::from_raw_handle(event.0) };
    let raw = HANDLE(event.as_raw_handle());
    // SAFETY: a live fence and event.
    unsafe { fence.SetEventOnCompletion(value, raw) }.map_err(api("SetEventOnCompletion"))?;
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let millis = u32::try_from(left.as_millis()).unwrap_or(u32::MAX - 1);
        // SAFETY: a live event; a bounded wait.
        let woken = unsafe { WaitForSingleObject(raw, millis) } == WAIT_OBJECT_0;
        // SAFETY: a live fence.
        if unsafe { fence.GetCompletedValue() } >= value {
            return Ok(());
        }
        if !woken || left.is_zero() {
            return Err(Error::Timeout(what));
        }
    }
}
