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

# roadmap
note: i dont know much about osdev stuff prefixed with ? is missing info and more stuff may be added in the feature

- [X] x86_64 basics
    - [X] GDT
    - [X] interrupts
    - [X] APIC
    - [X] framebuffer terminal
- [ ] keyboard
    - [X] ps/2 keyboard interrupt handling
    - [ ] usb keyboard handling
    - [X] keyboard driver
    - [X] key mapping
- [X] memory
    - [X] pagging
    - [X] kernel heap
    - [ ] more pagging
    - [ ] figure out how should i give apps memory?
    - [ ] bitmap?
    - [ ] pooling?
- [ ] fs
    - [ ] reading?
    - [ ] writing?
- [ ] networking
 - [ ] ?
- [ ] apps
    - [X] context switching, and simple processes
    - [ ] more advanced context switching
    - [ ] userspace
    - [ ] more about processes?
    - [ ] ELF support
    - [ ] wasm VM
    - [ ] more wasm
