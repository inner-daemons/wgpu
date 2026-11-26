//! How much memory wgpu-core's staging buffers take, and from which heap, while
//! many `queue.write_buffer` calls are pending. Vulkan only.

use std::fmt::Write as _;

use ash::vk;

use crate::{
    common::{MIB, NEW, Pair, Rec, VARIANTS, noise},
    info::vulkan,
};

const BENCH: &str = "bar_pressure";
const WRITE: u64 = 4 * MIB;

pub fn run(adapter: &wgpu::Adapter, pair: &Pair, rec: &mut Rec, out: &mut String) {
    if adapter.get_info().backend != wgpu::Backend::Vulkan {
        println!("  {BENCH}: skipped, Vulkan only");
        return;
    }
    let has_budget = unsafe { pair.ctxs[NEW].device.as_hal::<wgpu::hal::api::Vulkan>() }
        .is_some_and(|d| {
            d.enabled_device_extensions()
                .contains(&ash::ext::memory_budget::NAME)
        });
    if !has_budget {
        println!("  {BENCH}: skipped, needs VK_EXT_memory_budget");
        let _ = writeln!(out, "Skipped: the device has no VK_EXT_memory_budget.");
        let _ = writeln!(out);
        return;
    }
    let Some(memory) = vulkan::memory(adapter, pair) else {
        rec.check(
            BENCH,
            "memory properties",
            "-",
            &Err("couldn't read the Vulkan memory properties".into()),
        );
        return;
    };
    let map_write = memory
        .picks
        .iter()
        .find(|p| p.usage == "map_write")
        .unwrap();
    let mut heaps: Vec<u32> = map_write
        .types
        .iter()
        .filter_map(|&t| memory.heap_of(t))
        .collect();
    heaps.sort_unstable();
    heaps.dedup();

    let writes = if pair.quick() { 8 } else { 64 };
    let case_name = format!("{writes} pending writes of 4 MiB");
    println!("  {BENCH}: {case_name}");
    let data = noise((writes * WRITE) as usize, 99);
    let _ = writeln!(
        out,
        "Staging memory type: new {}, legacy {}.",
        memory.describe(map_write.types[NEW]),
        memory.describe(map_write.types[crate::common::LEGACY])
    );
    let _ = writeln!(out);

    for (v, ctx) in pair.ctxs.iter().enumerate() {
        let variant = VARIANTS[v];
        let dst = ctx.buffer(
            writes * WRITE,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        ctx.queue.submit(None);
        ctx.wait();

        let before = (heap_usage(adapter), ctx.device.generate_allocator_report());
        for k in 0..writes {
            let range = (k * WRITE) as usize..((k + 1) * WRITE) as usize;
            ctx.queue.write_buffer(&dst, k * WRITE, &data[range]);
        }
        let pending = (heap_usage(adapter), ctx.device.generate_allocator_report());
        ctx.queue.submit(None);
        ctx.wait();
        let after = (heap_usage(adapter), ctx.device.generate_allocator_report());

        for &heap in &heaps {
            let h = heap as usize;
            let case = format!("{case_name}, heap {heap}");
            let (Some(b), Some(p), Some(a)) = (&before.0, &pending.0, &after.0) else {
                continue;
            };
            let mib = |bytes: u64| bytes as f64 / MIB as f64;
            let delta = |x: u64, y: u64| (x as f64 - y as f64) / MIB as f64;
            rec.record(
                BENCH,
                &case,
                variant,
                "heap_size",
                mib(memory.heaps[h].size),
                "MiB",
            );
            rec.record(BENCH, &case, variant, "budget", mib(b.budget[h]), "MiB");
            rec.record(
                BENCH,
                &case,
                variant,
                "usage_before",
                mib(b.usage[h]),
                "MiB",
            );
            rec.record(
                BENCH,
                &case,
                variant,
                "usage_pending_delta",
                delta(p.usage[h], b.usage[h]),
                "MiB",
            );
            rec.record(
                BENCH,
                &case,
                variant,
                "usage_after_delta",
                delta(a.usage[h], b.usage[h]),
                "MiB",
            );
        }
        if let (Some(b), Some(p), Some(a)) = (&before.1, &pending.1, &after.1) {
            let case = format!("{case_name}, allocator");
            let mib = |bytes: u64| bytes as f64 / MIB as f64;
            for (metric, value) in [
                ("allocated_before", b.total_allocated_bytes),
                ("allocated_pending", p.total_allocated_bytes),
                ("allocated_after", a.total_allocated_bytes),
                ("reserved_before", b.total_reserved_bytes),
                ("reserved_pending", p.total_reserved_bytes),
                ("reserved_after", a.total_reserved_bytes),
            ] {
                rec.record(BENCH, &case, variant, metric, mib(value), "MiB");
            }
        }
        let result = ctx
            .error_check()
            .and_then(|()| ctx.check_contents(&dst, &data));
        rec.check(BENCH, &case_name, variant, &result);
    }
}

struct HeapUsage {
    usage: Vec<u64>,
    budget: Vec<u64>,
}

/// This process's usage and budget per heap, from `VK_EXT_memory_budget`.
fn heap_usage(adapter: &wgpu::Adapter) -> Option<HeapUsage> {
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
    let instance = hal.shared_instance().raw_instance();
    let mut budget = vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
    let count = {
        let mut properties = vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget);
        unsafe {
            instance
                .get_physical_device_memory_properties2(hal.raw_physical_device(), &mut properties)
        };
        properties.memory_properties.memory_heap_count as usize
    };
    Some(HeapUsage {
        usage: budget.heap_usage[..count].to_vec(),
        budget: budget.heap_budget[..count].to_vec(),
    })
}
