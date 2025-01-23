use kvm_ioctls::Kvm;
use kvm_bindings::kvm_userspace_memory_region;
use vm_memory::{mmap::GuestMemoryMmap, GuestAddress, GuestMemory};
use std::io;

fn main() -> Result<()> {
    /// Creates a new instance of the KVM (Kernel-based Virtual Machine) structure.
    /// This is typically used to interact with the KVM API for virtualization purposes.
    let kvm = Kvm::new()?;
    
    /// Creates a new virtual machine (VM) using the KVM (Kernel-based Virtual Machine) interface.
    let vm = kvm.create_vm()?;

    /// The size of the memory allocated for the virtual machine, set to 64 KB (0x10000 bytes).
    let mem_size = 0x10000; // 64 KB

    /// Creates a `GuestMemoryMmap` object with a single memory region starting at guest address 0
    /// and spanning `mem_size` bytes. This memory region is used to represent the guest's physical
    /// memory in the virtual machine.
    /// More deatils: /// More deatils here: https://github.com/rust-vmm/vm-memory/blob/main/DESIGN.md#backend-implementation-based-on-mmap
    let guest_memory: GuestMemoryMmap<()> = GuestMemoryMmap::from_ranges(&[(GuestAddress(0), mem_size)]).unwrap();

    /// Defines a `kvm_userspace_memory_region` structure to describe a memory region
    /// in the guest's physical address space. This structure includes the slot number,
    /// guest physical address, size of the memory region, userspace address, and flags.
    /// 
    /// - `slot`: The slot number for the memory region.
    /// - `guest_phys_addr`: The starting physical address in the guest's address space.
    /// - `memory_size`: The size of the memory region in bytes.
    /// - `userspace_addr`: The starting address of the memory region in the host's address space.
    /// - `flags`: Additional flags for the memory region (set to 0 in this case).
    let user_memory_region = kvm_userspace_memory_region {
        slot: 0,
        guest_phys_addr: 0,
        memory_size: mem_size as u64,
        userspace_addr: guest_memory.get_host_address(GuestAddress(0)).unwrap() as u64,
        flags: 0,
    };


    /// Sets the user memory region for the virtual machine using the `set_user_memory_region` method.
    /// This function takes a `kvm_userspace_memory_region` structure that describes the memory region
    /// in the guest's physical address space and maps it to the host's address space.
    unsafe { vm.set_user_memory_region(user_memory_region) }?;

    println!("Virtual machine created with {} KB memory", mem_size / 1024);
    Ok(())
}
