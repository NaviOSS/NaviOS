# NaviOS 
badly written open-source generic operating system made for fun written in rust!
i am attempting to make something like ChromeOS with native wasm support

# building and running
you need a linux system with bash, `xorriso`, `make`, `cargo` and `qemu-system-x86_64` to run do

```
cargo run
```
to build do
```
cargo build
```
this will make an iso `navios.iso`

currently using the [limine](https://limine-bootloader.org/) bootloader

# roadmap
note: i dont know much about osdev stuff prefixed with ? is missing info and more stuff may be added in the feature

- [X] x86_64 basics
    - [X] GDT
    - [X] interrupts
    - [X] APIC
    - [X] framebuffer terminal
- [X] framebuffer terminal
    - [X] scrolling
    - [ ] locking the terminal (threading, context switching)
    - [X] terminal shell process
- [ ] ACPI parsing
    - [X] RSDT parsing
    - [ ] XSDT parsing
    - [X] MADT parsing
    - [X] FADT parsing
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
    - [ ] ?
- [ ] GUI
    - [ ] ?
- [ ] apps
    - [X] context switching, and simple processes
    - [X] more advanced context switching, (pid, name, each process has it's own page table)
    - [ ] the ability for each process to have multiple threads
    - [ ] userspace
    - [ ] ELF support
    - [ ] wasm VM
    - [ ] more wasm
