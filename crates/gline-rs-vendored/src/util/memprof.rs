use cap::Cap;
use humansize::{format_size, DECIMAL};
use memory_stats::*;
use std::alloc::*;

#[global_allocator]
static ALLOCATOR: Cap<System> = Cap::new(System, usize::MAX);

pub fn print_memory_usage() {
    println!("Memory usage:");
    println!(
        "- Allocated: {} bytes ({})",
        ALLOCATOR.allocated(),
        format_size(ALLOCATOR.allocated(), DECIMAL)
    );
    if let Some(usage) = memory_stats() {
        println!(
            "- Physical: {} bytes ({})",
            usage.physical_mem,
            format_size(usage.physical_mem, DECIMAL)
        );
        println!(
            "- Virtual: {} bytes ({})",
            usage.virtual_mem,
            format_size(usage.virtual_mem, DECIMAL)
        );
    }
}
