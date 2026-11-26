#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MemoryPlacement {
    /// Device-local memory that the host does not map.
    DeviceOnly,
    /// Host-visible, write-combined memory for buffers the host only writes.
    HostWriteCombined,
    /// Host-visible, cached memory for buffers the host reads.
    HostCached,
}

pub(crate) fn buffer_placement(usage: wgt::BufferUses) -> MemoryPlacement {
    if usage.contains(wgt::BufferUses::MAP_READ) {
        MemoryPlacement::HostCached
    } else if usage.contains(wgt::BufferUses::MAP_WRITE) {
        MemoryPlacement::HostWriteCombined
    } else {
        MemoryPlacement::DeviceOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgt::BufferUses as U;

    #[test]
    fn buffer_placement_follows_map_usages() {
        assert_eq!(
            buffer_placement(U::STORAGE_READ_WRITE | U::COPY_DST),
            MemoryPlacement::DeviceOnly
        );
        assert_eq!(
            buffer_placement(U::MAP_WRITE | U::COPY_SRC),
            MemoryPlacement::HostWriteCombined
        );
        assert_eq!(
            buffer_placement(U::MAP_WRITE | U::STORAGE_READ_ONLY),
            MemoryPlacement::HostWriteCombined
        );
        assert_eq!(
            buffer_placement(U::MAP_READ | U::COPY_DST),
            MemoryPlacement::HostCached
        );
        assert_eq!(
            buffer_placement(U::MAP_READ | U::MAP_WRITE),
            MemoryPlacement::HostCached
        );
        assert_eq!(
            buffer_placement(U::MAP_READ | U::MAP_WRITE | U::STORAGE_READ_WRITE),
            MemoryPlacement::HostCached
        );
    }
}
