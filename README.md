# SafaOS
badly written open-source generic operating system made for fun written in rust!
i am attempting to make something like ChromeOS with native wasm support
this is my first OS!
**previously known as NaviOS**

**this project is written in rust and zig** which is inconvenience and expensive i know, but this was made for fun and learning purposes, even so our primary goal is the runtime results.
**star the repo!**

# building
you need: 
- bash
- git
- xorriso
- make
- cargo
- zig
simply run
```
cargo build
```

this should make an iso with path: `navios.iso` if successful
# running
to use the builtin cargo run feature you'll need:
- qemu-system-x86_64

and simply do
```
cargo run
```
or to run without kvm do
```
cargo run -- no-kvm
```
otherwise you have the iso feel free to do whatever you want with it

# current features:
- basics (x86_64: IDT, GDT, interrupts, ACPI, APIC, APIC keyboard, APIC timer, ...)
- pmm (bitmap allocator)
- buddy allocator
- generic keyboard driver
    - ps/2 scancode set 1 support
- basic ring0 framebuffer terminal
- scheduler with one-thread processes
- VFS with RamFS (ustar unpacking support)
- init ramdisk
- userspace:
    - resources
    - userspace elf executing
    - argc && argv
    - C libc written in zig (find it in libc/)
    - alot of syscalls (TODO: make a list)
    - program break and sbrk
    - init ramdisk with some programs written in zig (find it in bin/)

currently using the [limine](https://limine-bootloader.org/) bootloader

# structure
- `/build.rs` contains code that builds the final iso
- `/src` contains code that is executed on `cargo run` only runs the built iso using qemu for now
- `/kernel` contains the kernel code, written in rust
- `/kernel/src/arch/x86_64` x86_64 specific code such as syscalls, interrupts, x86_64 initing...
- `/libc` contains libc code written in zig
- `/bin` contains init ramdisk programs that is compiled on your os then copied to the init ramdisk in the final iso, written in zig
- `/macros` contains some additional rust proc macros to automatic some stuff...

# roadmap
note: i dont know much about osdev (this is my first OS), stuff prefixed with ? is missing info and more stuff may be added in the feature
## next:
- remove the bash requirement
- update README.md and all the README.mds and phillosophy.mds hiding in the repo
- rename NaviOS to SafaOS EVERYWHERE
- some bechmarking tools
    - PIT timer?

## roadmap
- [X] x86_64 basics
    - [X] GDT
    - [X] interrupts
    - [X] APIC
    - [X] framebuffer terminal
- [X] ACPI parsing
    - [X] RSDT parsing
    - [X] XSDT parsing
    - [X] MADT parsing
    - [X] FADT parsing
    - [ ] DSDT parsing (not planned, acpi sucks)
- [ ] ACPI powermangment
- [ ] keyboard
    - [X] ps/2 keyboard interrupt handling
    - [ ] usb keyboard handling
    - [X] keyboard driver
    - [X] key mapping
- [X] memory
    - [X] pagging
    - [X] kernel heap
    - [X] higher half kernel
- [ ] fs
    - [X] basic vfs
    - [X] ramfs
    - [ ] fat32
- [ ] networking?
    - [ ] OSI Model
        - [ ] Layer 1:
            - [ ] Manage Network Driver (NIC)
        - [ ] Layer 2:
            - [ ] Manage data from the layer 1:
                - [ ] NIC (Manage the physical network)
                - [ ] Ethernet frame 
                - [ ] ARP (For the ip adress)
        - [ ] Layer 3:
            - [ ] IPv4
            - [ ] ICMP (For commande like ping)
        - [ ] Layer 4:
            - [ ] TCP
            - [ ] UDP
- [ ] GUI?
- [ ] userspace
    - [X] context switching, and simple processes
    - [X] more advanced context switching, (pid, name, each process has it's own page table)
    - [X] process resources
    - [ ] the ability for each process to have multiple threads
    - [X] RING3
    - [X] ELF support
    - [ ] research wasm and multi-archicture support
