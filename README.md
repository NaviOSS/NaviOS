# NaviOS 
badly written open-source generic operating system made for fun written in rust!
i am attempting to make something like ChromeOS with native wasm support
this is my first OS!

**this project is written in rust and zig** which is inconvenience and expensive i know, but this was made for fun and learning purposes, even so our primary goal is the runtime results.
**star the repo!**

# building and running
you need a linux system with `bash`, `git`, `xorriso`, `make`, `cargo`, `zig` and `qemu-system-x86_64` (you need kvm otherwise run with `no-kvm` argument) to run do

```
cargo run
```
to build do
```
cargo build
```
this will make an iso `navios.iso`
to run without kvm do
```
cargo run -- no-kvm
```
# current features:
- basics (x86_64: IDT, GDT, interrupts, ACPI, APIC, APIC keyboard, APIC timer, ...)
- pmm (bitmap allocator)
- linked-list heap allocator
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

# roadmap
note: i dont know much about osdev (this is my first OS), stuff prefixed with ? is missing info and more stuff may be added in the feature
## next:
- [ ] sync
    - [ ] improve the peformance of the frame_allocator it takes 5 seconds to map 7*4 mbs without kvm?
- [ ] replace the current linked list allocator with a good buddy allocator
- [ ] remove the bash requirement
- [ ] update README.md

## roadmap
- [X] x86_64 basics
    - [X] GDT
    - [X] interrupts
    - [X] APIC
    - [X] framebuffer terminal
- [ ] framebuffer terminal
    - [X] scrolling
    - [X] locking the terminal (threading, context switching)
    - [ ] maybe try to RWLock the terminal instead of just locking the viewport?
    - [X] terminal shell process
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
    - [ ] a slab allocator
    - [ ] move more stuff to linked list and rely more on the slab allocator
    - [ ] figure out how should i give apps memory?
    - [X] higher half kernel
- [ ] fs
    - [X] basic vfs
    - [X] ramfs
    - [ ] fat32
- [ ] networking
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
- [ ] GUI
    - [ ] ?
- [ ] apps
    - [X] context switching, and simple processes
    - [X] more advanced context switching, (pid, name, each process has it's own page table)
    - [ ] process resources
    - [ ] the ability for each process to have multiple threads
    - [X] userspace
    - [X] ELF support
    - [ ] wasm VM
    - [ ] more wasm
