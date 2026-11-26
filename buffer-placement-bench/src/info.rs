//! Adapter memory layout and where each placement puts mappable buffers.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::common::{LEGACY, MIB, NEW, Pair, Rec, VARIANTS};

const BENCH: &str = "info";

pub fn run(adapter: &wgpu::Adapter, pair: &Pair, rec: &mut Rec, out: &mut String) -> Value {
    let info = adapter.get_info();
    let mut adapter_json = json!({
        "name": info.name,
        "backend": format!("{:?}", info.backend),
        "device_type": format!("{:?}", info.device_type),
        "driver": info.driver,
        "driver_info": info.driver_info,
        "vendor": info.vendor,
        "device": info.device,
    });
    let _ = writeln!(
        out,
        "- MAPPABLE_PRIMARY_BUFFERS: {}",
        pair.ctxs[NEW].mappable_primary_buffers()
    );
    let _ = writeln!(
        out,
        "- TIMESTAMP_QUERY: {}",
        adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
    );
    #[cfg(not(target_arch = "wasm32"))]
    vulkan::run(adapter, pair, rec, out, &mut adapter_json);
    #[cfg(windows)]
    dx12::run(pair, rec, out, &mut adapter_json);
    #[cfg(target_vendor = "apple")]
    metal::run(adapter, pair, rec, out, &mut adapter_json);
    let _ = writeln!(out);
    adapter_json
}

/// Checks a property of one buffer per variant that shows which placement the
/// device used.
#[cfg_attr(
    not(any(windows, target_vendor = "apple")),
    expect(dead_code, reason = "used by the DX12 and Metal sections")
)]
fn placement_usages() -> [(&'static str, wgpu::BufferUsages); 2] {
    [
        (
            "map_read_write",
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE,
        ),
        (
            "map_write",
            wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::STORAGE,
        ),
    ]
}

#[cfg(not(target_arch = "wasm32"))]
pub mod vulkan {
    use std::fmt::Write as _;

    use ash::vk;
    use serde_json::{Value, json};

    use super::{BENCH, LEGACY, MIB, NEW, Pair, Rec, VARIANTS};

    /// A copy of wgpu-hal's `MemoryPlacement`.
    #[derive(Clone, Copy, Debug)]
    enum Placement {
        DeviceOnly,
        HostWriteCombined,
        HostCached,
    }

    /// gpu-allocator's `MemoryLocation`, as trunk used it.
    #[derive(Clone, Copy, Debug)]
    enum Location {
        GpuOnly,
        CpuToGpu,
        GpuToCpu,
    }

    /// The buffer usages compared, with what each policy makes of them.
    const USAGES: [(&str, Placement, Location); 4] = [
        ("device_only", Placement::DeviceOnly, Location::GpuOnly),
        (
            "map_write",
            Placement::HostWriteCombined,
            Location::CpuToGpu,
        ),
        ("map_read", Placement::HostCached, Location::GpuToCpu),
        ("map_read_write", Placement::HostCached, Location::CpuToGpu),
    ];

    /// A copy of `select_memory_type` in `wgpu-hal/src/vulkan/memory.rs`.
    fn select_memory_type(
        types: &[vk::MemoryType],
        allowed_bits: u32,
        placement: Placement,
    ) -> Option<u32> {
        let host_coherent =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let required = match placement {
            Placement::DeviceOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            Placement::HostWriteCombined | Placement::HostCached => host_coherent,
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
                    Placement::DeviceOnly => (false, false),
                    Placement::HostWriteCombined => (device_local, cached),
                    Placement::HostCached => (!cached, device_local),
                };
                (rank, *index)
            })
            .map(|(index, _)| index as u32)
    }

    /// gpu-allocator 0.28's rule: the first allowed type with the preferred
    /// flags, else the first with the required flags. `CpuToGpu` also falls
    /// back to host memory when an allocation fails, which a static pick can't
    /// show.
    fn legacy_memory_type(
        types: &[vk::MemoryType],
        allowed_bits: u32,
        location: Location,
    ) -> Option<u32> {
        use vk::MemoryPropertyFlags as F;
        let (preferred, required) = match location {
            Location::GpuOnly => (F::DEVICE_LOCAL, F::DEVICE_LOCAL),
            Location::CpuToGpu => (
                F::HOST_VISIBLE | F::HOST_COHERENT | F::DEVICE_LOCAL,
                F::HOST_VISIBLE | F::HOST_COHERENT,
            ),
            Location::GpuToCpu => (
                F::HOST_VISIBLE | F::HOST_COHERENT | F::HOST_CACHED,
                F::HOST_VISIBLE | F::HOST_COHERENT,
            ),
        };
        let find = |flags: F| {
            types
                .iter()
                .enumerate()
                .find(|(i, ty)| allowed_bits & (1 << i) != 0 && ty.property_flags.contains(flags))
                .map(|(i, _)| i as u32)
        };
        find(preferred).or_else(|| find(required))
    }

    pub fn memory_flag_names(flags: vk::MemoryPropertyFlags) -> Vec<String> {
        use vk::MemoryPropertyFlags as F;
        let known = [
            (F::DEVICE_LOCAL, "DEVICE_LOCAL"),
            (F::HOST_VISIBLE, "HOST_VISIBLE"),
            (F::HOST_COHERENT, "HOST_COHERENT"),
            (F::HOST_CACHED, "HOST_CACHED"),
            (F::LAZILY_ALLOCATED, "LAZILY_ALLOCATED"),
            (F::PROTECTED, "PROTECTED"),
            (F::DEVICE_COHERENT_AMD, "DEVICE_COHERENT_AMD"),
            (F::DEVICE_UNCACHED_AMD, "DEVICE_UNCACHED_AMD"),
            (F::RDMA_CAPABLE_NV, "RDMA_CAPABLE_NV"),
        ];
        let mut names: Vec<String> = known
            .iter()
            .filter(|(f, _)| flags.contains(*f))
            .map(|(_, n)| (*n).to_owned())
            .collect();
        let rest = flags.as_raw() & !known.iter().fold(0, |a, (f, _)| a | f.as_raw());
        if rest != 0 {
            names.push(format!("{rest:#x}"));
        }
        names
    }

    fn heap_flag_names(flags: vk::MemoryHeapFlags) -> Vec<String> {
        use vk::MemoryHeapFlags as F;
        let known = [
            (F::DEVICE_LOCAL, "DEVICE_LOCAL"),
            (F::MULTI_INSTANCE, "MULTI_INSTANCE"),
        ];
        let mut names: Vec<String> = known
            .iter()
            .filter(|(f, _)| flags.contains(*f))
            .map(|(_, n)| (*n).to_owned())
            .collect();
        let rest = flags.as_raw() & !known.iter().fold(0, |a, (f, _)| a | f.as_raw());
        if rest != 0 {
            names.push(format!("{rest:#x}"));
        }
        names
    }

    pub struct Pick {
        pub usage: &'static str,
        /// Indexed by [`NEW`] and [`LEGACY`].
        pub types: [Option<u32>; 2],
    }

    pub struct Memory {
        pub device_type: vk::PhysicalDeviceType,
        pub types: Vec<vk::MemoryType>,
        pub heaps: Vec<vk::MemoryHeap>,
        pub buffer_bits: u32,
        pub allowed_bits: u32,
        pub picks: Vec<Pick>,
    }

    impl Memory {
        pub fn heap_of(&self, ty: Option<u32>) -> Option<u32> {
            ty.map(|t| self.types[t as usize].heap_index)
        }

        pub fn describe(&self, ty: Option<u32>) -> String {
            match ty {
                None => "none".into(),
                Some(t) => {
                    let ty = self.types[t as usize];
                    format!(
                        "{t} ({}, heap {})",
                        memory_flag_names(ty.property_flags).join(" | "),
                        ty.heap_index
                    )
                }
            }
        }
    }

    /// Reads the memory properties and works out each policy's picks for a
    /// real `STORAGE | TRANSFER_SRC | TRANSFER_DST` buffer.
    pub fn memory(adapter: &wgpu::Adapter, pair: &Pair) -> Option<Memory> {
        let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
        let instance = hal.shared_instance().raw_instance();
        let physical_device = hal.raw_physical_device();
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let memory = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let types = memory.memory_types[..memory.memory_type_count as usize].to_vec();
        let heaps = memory.memory_heaps[..memory.memory_heap_count as usize].to_vec();

        let device = unsafe { pair.ctxs[NEW].device.as_hal::<wgpu::hal::api::Vulkan>() }?;
        let raw = device.raw_device();
        let buffer = unsafe {
            raw.create_buffer(
                &vk::BufferCreateInfo::default()
                    .size(MIB)
                    .usage(
                        vk::BufferUsageFlags::STORAGE_BUFFER
                            | vk::BufferUsageFlags::TRANSFER_SRC
                            | vk::BufferUsageFlags::TRANSFER_DST,
                    )
                    .sharing_mode(vk::SharingMode::EXCLUSIVE),
                None,
            )
        }
        .ok()?;
        let requirements = unsafe { raw.get_buffer_memory_requirements(buffer) };
        unsafe { raw.destroy_buffer(buffer, None) };

        // wgpu-hal's `valid_ash_memory_types`.
        let known = vk::MemoryPropertyFlags::DEVICE_LOCAL
            | vk::MemoryPropertyFlags::HOST_VISIBLE
            | vk::MemoryPropertyFlags::HOST_COHERENT
            | vk::MemoryPropertyFlags::HOST_CACHED
            | vk::MemoryPropertyFlags::LAZILY_ALLOCATED;
        let valid = types
            .iter()
            .enumerate()
            .filter(|(_, ty)| known.contains(ty.property_flags))
            .fold(0u32, |bits, (i, _)| bits | (1 << i));
        let allowed_bits = requirements.memory_type_bits & valid;
        let picks = USAGES
            .iter()
            .map(|&(usage, placement, location)| Pick {
                usage,
                types: [
                    select_memory_type(&types, allowed_bits, placement),
                    legacy_memory_type(&types, allowed_bits, location),
                ],
            })
            .collect();
        Some(Memory {
            device_type: properties.device_type,
            types,
            heaps,
            buffer_bits: requirements.memory_type_bits,
            allowed_bits,
            picks,
        })
    }

    pub fn run(
        adapter: &wgpu::Adapter,
        pair: &Pair,
        rec: &mut Rec,
        out: &mut String,
        json: &mut Value,
    ) {
        if adapter.get_info().backend != wgpu::Backend::Vulkan {
            return;
        }
        let Some(memory) = memory(adapter, pair) else {
            let _ = writeln!(out, "- couldn't read the Vulkan memory properties");
            rec.check(
                BENCH,
                "memory properties",
                "-",
                &Err("couldn't read the Vulkan memory properties".into()),
            );
            return;
        };
        let _ = writeln!(out, "- Vulkan device type: {:?}", memory.device_type);
        for (i, heap) in memory.heaps.iter().enumerate() {
            let _ = writeln!(
                out,
                "- memory heap {i}: {} MiB, {}",
                heap.size / MIB,
                heap_flag_names(heap.flags).join(" | ")
            );
            let case = format!("heap {i}");
            rec.record(
                BENCH,
                &case,
                "-",
                "size",
                heap.size as f64 / MIB as f64,
                "MiB",
            );
            rec.record(
                BENCH,
                &case,
                "-",
                "device_local",
                heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) as u32 as f64,
                "bool",
            );
        }
        for (i, ty) in memory.types.iter().enumerate() {
            let _ = writeln!(
                out,
                "- memory type {i}: heap {}, {}",
                ty.heap_index,
                memory_flag_names(ty.property_flags).join(" | ")
            );
            let case = format!("type {i}");
            rec.record(BENCH, &case, "-", "heap", ty.heap_index as f64, "index");
            rec.record(
                BENCH,
                &case,
                "-",
                "flags",
                ty.property_flags.as_raw() as f64,
                "bits",
            );
        }
        let _ = writeln!(
            out,
            "- memoryTypeBits of a STORAGE | TRANSFER_SRC | TRANSFER_DST buffer: {:#x} \
             ({:#x} after dropping types with unknown flags)",
            memory.buffer_bits, memory.allowed_bits
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "| usage | new | legacy |");
        let _ = writeln!(out, "|---|---|---|");
        for pick in &memory.picks {
            let _ = writeln!(
                out,
                "| {} | {} | {} |",
                pick.usage,
                memory.describe(pick.types[NEW]),
                memory.describe(pick.types[LEGACY])
            );
            for v in [NEW, LEGACY] {
                let index = pick.types[v].map_or(-1.0, |t| t as f64);
                let heap = memory.heap_of(pick.types[v]).map_or(-1.0, |h| h as f64);
                rec.record(
                    BENCH,
                    pick.usage,
                    VARIANTS[v],
                    "memory_type",
                    index,
                    "index",
                );
                rec.record(BENCH, pick.usage, VARIANTS[v], "heap", heap, "index");
            }
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "Legacy picks follow gpu-allocator's static rule; its CpuToGpu fallback to host \
             memory when an allocation fails isn't shown."
        );

        json["vulkan"] = json!({
            "device_type": format!("{:?}", memory.device_type),
            "memory_heaps": memory.heaps.iter().map(|h| json!({
                "size": h.size,
                "flags": heap_flag_names(h.flags),
            })).collect::<Vec<_>>(),
            "memory_types": memory.types.iter().map(|t| json!({
                "flags": memory_flag_names(t.property_flags),
                "heap": t.heap_index,
            })).collect::<Vec<_>>(),
            "buffer_memory_type_bits": memory.buffer_bits,
            "allowed_memory_type_bits": memory.allowed_bits,
            "picks": memory.picks.iter().map(|p| (p.usage.to_owned(), json!({
                "new": p.types[NEW],
                "legacy": p.types[LEGACY],
            }))).collect::<serde_json::Map<_, _>>(),
        });
    }
}

#[cfg(windows)]
mod dx12 {
    use std::fmt::Write as _;

    use serde_json::{Value, json};
    use windows::Win32::Graphics::Direct3D12 as d3d12;

    use super::{BENCH, Pair, Rec, VARIANTS, placement_usages};

    pub fn run(pair: &Pair, rec: &mut Rec, out: &mut String, json: &mut Value) {
        let Some(device) = (unsafe { pair.ctxs[0].device.as_hal::<wgpu::hal::api::Dx12>() }) else {
            return;
        };
        let raw = device.raw_device();
        let mut architecture = d3d12::D3D12_FEATURE_DATA_ARCHITECTURE1::default();
        let architecture_ok = unsafe {
            raw.CheckFeatureSupport(
                d3d12::D3D12_FEATURE_ARCHITECTURE1,
                <*mut _>::cast(&mut architecture),
                size_of_val(&architecture) as u32,
            )
        }
        .is_ok();
        let mut options16 = d3d12::D3D12_FEATURE_DATA_D3D12_OPTIONS16::default();
        let options16_ok = unsafe {
            raw.CheckFeatureSupport(
                d3d12::D3D12_FEATURE_D3D12_OPTIONS16,
                <*mut _>::cast(&mut options16),
                size_of_val(&options16) as u32,
            )
        }
        .is_ok();
        let uma = architecture_ok.then(|| architecture.UMA.as_bool());
        let coherent = architecture_ok.then(|| architecture.CacheCoherentUMA.as_bool());
        let upload_heap = options16_ok.then(|| options16.GPUUploadHeapSupported.as_bool());
        for (name, value) in [
            ("UMA", uma),
            ("CacheCoherentUMA", coherent),
            ("GPUUploadHeapSupported", upload_heap),
        ] {
            let _ = writeln!(
                out,
                "- {name}: {}",
                value.map_or("unknown".into(), |v| v.to_string())
            );
            if let Some(v) = value {
                rec.record(BENCH, "adapter", "-", name, v as u32 as f64, "bool");
            }
        }
        drop(device);
        json["dx12"] = json!({
            "uma": uma,
            "cache_coherent_uma": coherent,
            "gpu_upload_heap_supported": upload_heap,
        });

        // The heap each device put a buffer in shows that the legacy switch
        // took effect.
        let mut heaps = serde_json::Map::new();
        for (usage_name, usage) in placement_usages() {
            for (v, ctx) in pair.ctxs.iter().enumerate() {
                if !ctx.mappable_primary_buffers() {
                    continue;
                }
                let buffer = ctx.buffer(4096, usage);
                let Some(hal) = (unsafe { buffer.as_hal::<wgpu::hal::api::Dx12>() }) else {
                    continue;
                };
                let mut properties = d3d12::D3D12_HEAP_PROPERTIES::default();
                if unsafe {
                    hal.raw_resource()
                        .GetHeapProperties(Some(&mut properties), None)
                }
                .is_err()
                {
                    let _ = writeln!(
                        out,
                        "- {usage_name} {}: GetHeapProperties failed",
                        VARIANTS[v]
                    );
                    continue;
                }
                let write_combined = properties.Type == d3d12::D3D12_HEAP_TYPE_UPLOAD
                    || properties.CPUPageProperty == d3d12::D3D12_CPU_PAGE_PROPERTY_WRITE_COMBINE;
                let _ = writeln!(
                    out,
                    "- {usage_name} buffer, {}: heap type {}, CPU page property {}",
                    VARIANTS[v], properties.Type.0, properties.CPUPageProperty.0
                );
                rec.record(
                    BENCH,
                    usage_name,
                    VARIANTS[v],
                    "heap_type",
                    properties.Type.0 as f64,
                    "enum",
                );
                rec.record(
                    BENCH,
                    usage_name,
                    VARIANTS[v],
                    "cpu_page_property",
                    properties.CPUPageProperty.0 as f64,
                    "enum",
                );
                heaps.insert(
                    format!("{usage_name}/{}", VARIANTS[v]),
                    json!({"type": properties.Type.0, "cpu_page_property": properties.CPUPageProperty.0}),
                );
                // MAP_WRITE alone is write-combined either way; MAP_READ |
                // MAP_WRITE only with the legacy placement.
                let want = usage_name == "map_write" || v == super::LEGACY;
                let result = if write_combined == want {
                    Ok(())
                } else {
                    Err(format!(
                        "write-combined is {write_combined}, expected {want}"
                    ))
                };
                rec.check(
                    BENCH,
                    &format!("placement {usage_name}"),
                    VARIANTS[v],
                    &result,
                );
            }
        }
        json["dx12"]["buffer_heaps"] = Value::Object(heaps);
    }
}

#[cfg(target_vendor = "apple")]
mod metal {
    use std::fmt::Write as _;

    use objc2_metal::{MTLCPUCacheMode, MTLDevice as _, MTLResource as _};
    use serde_json::{Value, json};

    use super::{BENCH, LEGACY, MIB, Pair, Rec, VARIANTS, placement_usages};

    pub fn run(
        adapter: &wgpu::Adapter,
        pair: &Pair,
        rec: &mut Rec,
        out: &mut String,
        json: &mut Value,
    ) {
        let Some(hal) = (unsafe { adapter.as_hal::<wgpu::hal::api::Metal>() }) else {
            return;
        };
        let device = hal.raw_device();
        let unified = device.hasUnifiedMemory();
        let working_set = device.recommendedMaxWorkingSetSize();
        drop(hal);
        let _ = writeln!(out, "- hasUnifiedMemory: {unified}");
        let _ = writeln!(
            out,
            "- recommendedMaxWorkingSetSize: {} MiB",
            working_set / MIB
        );
        rec.record(
            BENCH,
            "adapter",
            "-",
            "hasUnifiedMemory",
            unified as u32 as f64,
            "bool",
        );
        let mut modes = serde_json::Map::new();

        // The cache mode each device gave a buffer shows that the legacy
        // switch took effect.
        for (usage_name, usage) in placement_usages() {
            for (v, ctx) in pair.ctxs.iter().enumerate() {
                if !ctx.mappable_primary_buffers() {
                    continue;
                }
                let buffer = ctx.buffer(4096, usage);
                let Some(hal) = (unsafe { buffer.as_hal::<wgpu::hal::api::Metal>() }) else {
                    continue;
                };
                let write_combined =
                    hal.raw_handle().cpuCacheMode() == MTLCPUCacheMode::WriteCombined;
                let _ = writeln!(
                    out,
                    "- {usage_name} buffer, {}: write-combined {write_combined}",
                    VARIANTS[v]
                );
                rec.record(
                    BENCH,
                    usage_name,
                    VARIANTS[v],
                    "write_combined",
                    write_combined as u32 as f64,
                    "bool",
                );
                modes.insert(
                    format!("{usage_name}/{}", VARIANTS[v]),
                    Value::Bool(write_combined),
                );
                // MAP_WRITE alone is write-combined either way; MAP_READ |
                // MAP_WRITE only with the legacy placement.
                let want = usage_name == "map_write" || v == LEGACY;
                let result = if write_combined == want {
                    Ok(())
                } else {
                    Err(format!(
                        "write-combined is {write_combined}, expected {want}"
                    ))
                };
                rec.check(
                    BENCH,
                    &format!("placement {usage_name}"),
                    VARIANTS[v],
                    &result,
                );
            }
        }
        json["metal"] = json!({
            "has_unified_memory": unified,
            "recommended_max_working_set_size": working_set,
            "buffer_write_combined": modes,
        });
    }
}
