//! Role: GPU texture sharing events (feature `gpu-interop`, experimental).

use kivact_reactive::Runtime;
use kivact_types::{ExternalImageId, ExternalImport, GpuAdapter};

pub(super) fn external_imported(rt: &Runtime, image: ExternalImageId, result: &ExternalImport) {
    crate::gpu_interop::imported(rt, image, result);
}

pub(super) fn adapter_changed(rt: &Runtime, adapter: Option<&GpuAdapter>) {
    crate::gpu_interop::adapter_changed(rt, adapter);
}
