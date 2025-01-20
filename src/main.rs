use kvm_ioctls::Kvm;
use kvm_bindings::kvm_userspace_memory_region;
use vm_memory::{mmap::GuestMemoryMmap, GuestAddress, GuestMemory};
use std::io;

fn main() {
    // Initialize KVM
    /// Creates a new instance of the KVM (Kernel-based Virtual Machine) structure.
    /// This is typically used to interact with the KVM API for virtualization purposes.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the KVM instance cannot be created. This could happen
    /// due to insufficient permissions or if the KVM module is not loaded.
    let kvm = Kvm::new().unwrap();
    /// Creates a new virtual machine (VM) using the KVM (Kernel-based Virtual Machine) interface.
    /// 
    /// This function initializes a new VM instance by calling the `create_vm` method on the KVM object.
    /// If the VM creation fails, the function will panic and unwrap the error.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the VM creation fails.
    let vm = kvm.create_vm().unwrap();

    /// The size of the memory allocated for the virtual machine, set to 64 KB (0x10000 bytes).
    let mem_size = 0x10000; // 64 KB

    /// Creates a `GuestMemoryMmap` object with a single memory region starting at guest address 0
    /// and spanning `mem_size` bytes. This memory region is used to represent the guest's physical
    /// memory in the virtual machine. The `unwrap()` call ensures that the creation of the memory
    /// region is successful, and will panic if it fails.
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
    /// in the guest's physical address space and maps it to the host's address space. The `unsafe` block
    /// is required because this operation involves raw pointers and direct memory manipulation, which
    /// can lead to undefined behavior if not handled correctly.
    unsafe { vm.set_user_memory_region(user_memory_region) }.unwrap();

    println!("Virtual machine created with {} KB memory", mem_size / 1024);
}
