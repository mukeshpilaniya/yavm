use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{ioctls::vm::VmFd, Kvm, VcpuFd};
use std::f32::consts::E;
use std::io;
use std::sync::Arc;
use vm_memory::guest_memory;
use vm_memory::{mmap::GuestMemoryMmap, GuestAddress, GuestMemory};

const ZEROPG_START: u64 = 0x7000;

fn setup_vcpu(
    vm_fd: &Arc<VmFd>,
    kernel_offset: GuestAddress,
) -> Result<VcpuFd, Box<dyn std::error::Error>> {
    /// The number of virtual CPUs (vCPUs) to create for the virtual machine.
    let vcpu_count = 1;

    /// Creates a new KVM vCPU file descriptor and maps the memory corresponding its kvm_run structure.
    let vcpu_fd = vm_fd.create_vcpu(0)?;

    /// Sets the vCPU registers using the `set_regs` method.
    /// This function takes a `kvm_regs` structure that describes the vCPU registers.
    /// The `set_regs` method is used to set the general-purpose registers (GPRs) of the vCPU.
    /// The `rip` register is set to the starting address of the kernel image, which is the entry point of the kernel.
    let mut regs = vcpu_fd.get_regs()?;
    regs.rip = kernel_offset.0;
    regs.rflagsv = 0x0000_0000_0000_0002u64;
    regs.rsi = ZEROPG_START;
    vcpu_fd.set_regs(&regs)?;

    Ok(vcpu_fd)
}

fn load_kernel(
    guest_mem: GuestMemory,
    kernel_offset: Option<GuestAddress>,
) -> Result<(), Box<dyn std::error::Error>> {
    /// The path to the kernel image file.
    let kernel_path = "bzImage";

    /// Opens the kernel image file in read-only mode.
    let kernel_file = std::fs::File::open(kernel_path)?;
    let highmem_start_address = None;

    // linux_loader::loader::KernelLoader::load(
    //     guest_mem,
    //     kernel_offset,
    //     kernel_image,
    //     highmem_start_address,
    // );
    linux_loader::loader::elf::Elf::load(
        &guest_mem,
        kernel_offset,
        kernel_image,
        highmem_start_address,
    )?;

    Ok(())
}

fn setup_memory(vm_fd: &Arc<VmFd>) -> Result<GuestMemoryMmap, Box<dyn std::error::Error>> {
    /// The size of the memory allocated for the virtual machine, set to 1MB KB (0x100000 bytes).
    let mem_size = 0x100000; // 1 MB

    /// The starting address of the memory region in the guest's physical address space.
    let guest_addr = GuestAddress(0);

    /// Creates a `GuestMemoryMmap` object with a single memory region starting at guest address 0
    /// and spanning `mem_size` bytes. This memory region is used to represent the entire guest's physical
    /// memory in the virtual machine.
    /// More deatils: /// More deatils here: https://github.com/rust-vmm/vm-memory/blob/main/DESIGN.md#backend-implementation-based-on-mmap
    /// Internaly it's calling mmap system call to map the memory region.
    ///  unsafe {
    // `libc::mmap(
    //     null_mut(),
    //     size,
    //     prot,
    //     flags,
    //     fd,
    //     offset as libc::off_t,
    // )`
    let guest_memory: GuestMemoryMmap<()> =
        GuestMemoryMmap::from_ranges(&[(guest_addr, mem_size)])?;

    /// Register the memory region with the KVM.
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
        // first slot is 0, because we are using only one memory region.
        slot: 0,
        guest_phys_addr: guest_addr.0 as u64,
        memory_size: mem_size as u64,
        // Get the host virtual address corresponding to the guest address.
        userspace_addr: guest_memory.get_host_address(GuestAddress(0)).unwrap() as u64,
        flags: 0,
    };

    /// Sets the user memory region for the virtual machine using the `set_user_memory_region` method.
    /// This function takes a `kvm_userspace_memory_region` structure that describes the memory region
    /// in the guest's physical address space and maps it to the host's address space.
    unsafe { vm_fd.set_user_memory_region(user_memory_region) }?;
    Ok(guest_memory)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Creates a new instance of the KVM (Kernel-based Virtual Machine) structure.
    /// This is typically used to interact with the KVM API for virtualization purposes.
    let kvm = Kvm::new().map_err(Error::kvm_ioctls)?;

    /// Retrieves the KVM API version supported by the host system.
    let kvm_api_ver = kvm.get_api_version();
    info!("KVM API version: {}", kvm_api_ver);

    // Create a VM file descriptor
    let vm_fd = Arc::new(kvm.create_vm()?);

    // Setup memory for the VM
    let guest_mem = setup_memory(&vm_fd)?;

    // kernel load address
    let kernel_offset = GuestAddress(0);
    load_kernel(guest_mem, Some(kernel_offset))?;

    //let vcpu_fd = vm_fd.create_vcpu(0)?;
    let vcpu_fd = setup_vcpu(&vm_fd, kernel_offset)?;

    println!("Virtual machine created with {} KB memory", mem_size / 1024);

    // Run the VCPU
    loop {
        match vcpu_fd.run()? {
            kvm_ioctls::VcpuExit::Hlt => break,
            kvm_ioctls::VcpuExit::MmioRead(addr, data) => {
                // Handle MMIO read
            }
            kvm_ioctls::VcpuExit::MmioWrite(addr, data) => {
                // Handle MMIO write
            }
            kvm_ioctls::VcpuExit::IoIn(addr, data) => {
                // Handle I/O port read
            }
            kvm_ioctls::VcpuExit::IoOut(addr, data) => {
                // Handle I/O port write
            }
            kvm_ioctls::VcpuExit::Shutdown => break,
            _ => continue,
        }
    }

    Ok(())
}
