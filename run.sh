#!/bin/bash
make
export qemu=qemu-system-i386
$qemu -kernel kernel.bin -display gtk &

echo "qemu starting..."
sleep 2
echo "done"

while true; do read -p "type exit to exit: " </dev/tty;  if [ "$REPLY" == "exit" ]; then break; fi; done
killall $qemu