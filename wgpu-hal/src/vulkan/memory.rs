//! Device memory sub-allocation.
//!
//! gpu-allocator frees a block only after every allocation in it is freed,
//! so short-lived resources get their own pool.

use alloc::vec::Vec;
use core::ops::Deref;

use ash::vk;
use wgpu_sync::Mutex;

use crate::auxil::memory::MemoryPlacement;

#[derive(Debug, Clone, Copy)]
pub(super) enum MemoryPool {
    General,
    Transient,
}

impl MemoryPool {
    pub(super) fn from_memory_flags(flags: crate::MemoryFlags) -> Self {
        if flags.contains(crate::MemoryFlags::TRANSIENT) {
            Self::Transient
        } else {
            Self::General
        }
    }
}

/// A sub-allocation that records which pool to free it to.
#[derive(Debug)]
pub struct MemoryAllocation {
    pool: MemoryPool,
    inner: gpu_allocator::vulkan::Allocation,
}

impl Deref for MemoryAllocation {
    type Target = gpu_allocator::vulkan::Allocation;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(super) struct MemoryAllocators {
    general: Mutex<gpu_allocator::vulkan::Allocator>,
    transient: Mutex<gpu_allocator::vulkan::Allocator>,
}

impl MemoryAllocators {
    pub(super) fn new(
        instance: ash::Instance,
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        buffer_device_address: bool,
        allocation_sizes: crate::AllocationSizes,
    ) -> Result<Self, gpu_allocator::AllocationError> {
        let mut desc = gpu_allocator::vulkan::AllocatorCreateDesc {
            instance,
            device,
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address,
            allocation_sizes: allocation_sizes.into(),
        };
        let general = gpu_allocator::vulkan::Allocator::new(&desc)?;
        desc.allocation_sizes = allocation_sizes.transient().into();
        let transient = gpu_allocator::vulkan::Allocator::new(&desc)?;
        Ok(Self {
            general: Mutex::new(general),
            transient: Mutex::new(transient),
        })
    }

    fn allocator(&self, pool: MemoryPool) -> &Mutex<gpu_allocator::vulkan::Allocator> {
        match pool {
            MemoryPool::General => &self.general,
            MemoryPool::Transient => &self.transient,
        }
    }

    pub(super) fn allocate(
        &self,
        pool: MemoryPool,
        desc: &gpu_allocator::vulkan::AllocationCreateDesc<'_>,
    ) -> Result<MemoryAllocation, gpu_allocator::AllocationError> {
        let inner = self.allocator(pool).lock().allocate(desc)?;
        Ok(MemoryAllocation { pool, inner })
    }

    pub(super) fn free(
        &self,
        allocation: MemoryAllocation,
    ) -> Result<(), gpu_allocator::AllocationError> {
        self.allocator(allocation.pool)
            .lock()
            .free(allocation.inner)
    }

    pub(super) fn generate_report(&self) -> wgt::AllocatorReport {
        let mut allocations = Vec::new();
        let mut blocks = Vec::new();
        let mut total_allocated_bytes = 0;
        let mut total_reserved_bytes = 0;

        for allocator in [&self.general, &self.transient] {
            let report = allocator.lock().generate_report();

            // Each block's range indexes into its own report's allocation list.
            let allocation_base = allocations.len();
            allocations.extend(
                report
                    .allocations
                    .into_iter()
                    .map(|alloc| wgt::AllocationReport {
                        name: alloc.name,
                        offset: alloc.offset,
                        size: alloc.size,
                    }),
            );
            blocks.extend(
                report
                    .blocks
                    .into_iter()
                    .map(|block| wgt::MemoryBlockReport {
                        size: block.size,
                        allocations: (block.allocations.start + allocation_base)
                            ..(block.allocations.end + allocation_base),
                    }),
            );
            total_allocated_bytes += report.total_allocated_bytes;
            total_reserved_bytes += report.total_capacity_bytes;
        }

        wgt::AllocatorReport {
            allocations,
            blocks,
            total_allocated_bytes,
            total_reserved_bytes,
        }
    }
}

/// Picks the memory type for `placement` among those in `allowed_bits`.
///
/// Returns `None` if no allowed type has the required flags.
pub(super) fn select_memory_type(
    types: &[vk::MemoryType],
    allowed_bits: u32,
    placement: MemoryPlacement,
) -> Option<u32> {
    let host_coherent =
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let required = match placement {
        MemoryPlacement::DeviceOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        MemoryPlacement::HostWriteCombined | MemoryPlacement::HostCached => host_coherent,
    };
    types
        .iter()
        .enumerate()
        .filter(|(index, ty)| {
            allowed_bits & (1 << index) != 0 && ty.property_flags.contains(required)
        })
        .min_by_key(|(index, ty)| {
            let device_local = ty
                .property_flags
                .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let cached = ty
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_CACHED);
            let rank = match placement {
                MemoryPlacement::DeviceOnly => (false, false),
                MemoryPlacement::HostWriteCombined => (device_local, cached),
                MemoryPlacement::HostCached => (!cached, device_local),
            };
            (rank, *index)
        })
        .map(|(index, _)| index as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auxil::memory::MemoryPlacement as P;

    const DL: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    const HV: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::HOST_VISIBLE;
    const HC: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::HOST_COHERENT;
    const CA: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::HOST_CACHED;

    fn ty(flags: vk::MemoryPropertyFlags, heap: u32) -> vk::MemoryType {
        vk::MemoryType {
            property_flags: flags,
            heap_index: heap,
        }
    }

    fn select_all(types: &[vk::MemoryType]) -> (Option<u32>, Option<u32>, Option<u32>) {
        (
            select_memory_type(types, u32::MAX, P::DeviceOnly),
            select_memory_type(types, u32::MAX, P::HostWriteCombined),
            select_memory_type(types, u32::MAX, P::HostCached),
        )
    }

    fn nvidia_no_rebar() -> [vk::MemoryType; 5] {
        [
            ty(vk::MemoryPropertyFlags::empty(), 1),
            ty(DL, 0),
            ty(HV | HC, 1),
            ty(HV | HC | CA, 1),
            ty(DL | HV | HC, 2),
        ]
    }

    #[test]
    fn select_nvidia_no_rebar() {
        assert_eq!(select_all(&nvidia_no_rebar()), (Some(1), Some(2), Some(3)));
    }

    #[test]
    fn select_radv_rebar() {
        let types = [
            ty(DL, 0),
            ty(HV | HC, 1),
            ty(DL | HV | HC, 0),
            ty(HV | HC | CA, 1),
        ];
        assert_eq!(select_all(&types), (Some(0), Some(1), Some(3)));
    }

    #[test]
    fn select_radv_apu() {
        let types = [
            ty(DL, 0),
            ty(HV | HC, 1),
            ty(DL | HV | HC, 0),
            ty(HV | HC | CA, 1),
        ];
        assert_eq!(select_all(&types), (Some(0), Some(1), Some(3)));
    }

    #[test]
    fn select_anv() {
        let types = [ty(DL, 0), ty(DL | HV | HC | CA, 0), ty(DL | HV | HC, 0)];
        assert_eq!(select_all(&types), (Some(0), Some(2), Some(1)));
    }

    #[test]
    fn select_adreno() {
        let types = [
            ty(DL, 0),
            ty(DL | HV | HC, 0),
            ty(DL | HV | CA, 0),
            ty(DL | HV | HC | CA, 0),
        ];
        assert_eq!(select_all(&types), (Some(0), Some(1), Some(3)));
    }

    #[test]
    fn select_lavapipe() {
        let types = [ty(DL | HV | HC | CA, 0)];
        assert_eq!(select_all(&types), (Some(0), Some(0), Some(0)));
    }

    #[test]
    fn respects_allowed_bits() {
        assert_eq!(
            select_memory_type(&nvidia_no_rebar(), !(1 << 2), P::HostWriteCombined),
            Some(3)
        );
    }

    #[test]
    fn none_without_candidates() {
        let types = nvidia_no_rebar();
        assert_eq!(
            select_memory_type(&types, 1 << 1, P::HostWriteCombined),
            None
        );
        assert_eq!(select_memory_type(&types, 1 << 1, P::HostCached), None);
    }
}
