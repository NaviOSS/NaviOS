# NaviOS 
badly written open-source generic operating system made for fun written in rust!
i am attempting to make something like ChromeOS with native wasm support

# building and running
you simply need `cargo` and qemu-system-x86_64 to run do
```
cargo run
```
to build do
```
cargo build
```
for now am using a crate called `bootloader` which i dont understands much about it is a bootloader it provides bootinfo and it also builds in os image using cargo TODO figure out more and maybe switch to my own bootloader or another one
# next
- kernel is getting complex i must do:
    - globals, put alot of tought about that, one kernel struct? thread safety?
    - overall improvments to the code
    - make the code not spaghetti
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
    - [ ] figure out how should i give apps memory?
    - [X] higher half kernel
- [ ] fs
    - [X] basic vfs
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
