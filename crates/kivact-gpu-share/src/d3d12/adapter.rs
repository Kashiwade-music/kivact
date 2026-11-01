//! Role: Finds the DXGI adapter of a GpuAdapter by its LUID.

use crate::common::{Error, api};
use crate::d3d12_contract::luid_bytes;
use kivact_types::{GpuAdapter, GpuBackend};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, DXGI_CREATE_FACTORY_FLAGS, IDXGIAdapter1, IDXGIFactory4,
};

/// The LUID `adapter` reports for Direct3D 12.
pub(crate) fn luid(adapter: &GpuAdapter) -> Result<[u8; 8], Error> {
    if adapter.backend != GpuBackend::Dx12 {
        return Err(Error::Mismatch(format!(
            "the window's device is {:?}, not Direct3D 12",
            adapter.backend
        )));
    }
    adapter
        .luid
        .ok_or_else(|| Error::Mismatch("the adapter has no LUID".into()))
}

/// The DXGI adapter that is `adapter` (its LUID), to make the engine's
/// device on.
pub fn find_adapter(adapter: &GpuAdapter) -> Result<IDXGIAdapter1, Error> {
    let wanted = luid(adapter)?;
    // SAFETY: plain factory creation.
    let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) }
        .map_err(api("CreateDXGIFactory2"))?;
    for i in 0.. {
        // SAFETY: enumerating until the factory says there are no more.
        let Ok(found) = (unsafe { factory.EnumAdapters1(i) }) else {
            break;
        };
        // SAFETY: a live adapter.
        let desc = unsafe { found.GetDesc1() }.map_err(api("GetDesc1"))?;
        if luid_bytes(desc.AdapterLuid) == wanted {
            return Ok(found);
        }
    }
    Err(Error::NoAdapter)
}
