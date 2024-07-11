#!/bin/bash
cargo build
export qemu=qemu-system-i386
$qemu -kernel target/x86-navi/debug/NaviOS.elf &

echo "qemu starting..."
sleep 2
echo "done"

while true; do read -p "type exit to exit: " </dev/tty;  if [ "$REPLY" == "exit" ]; then break; fi; done
killall $qemu