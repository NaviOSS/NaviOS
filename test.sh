#!/bin/bash
# This script simply runs the OS with qemu, no-gui, and no-kvm then checks if the serial output 
# contains a successful output (returns 0) or a kernel panic (returns 1)

cargo run -- no-kvm no-gui > TEST.log.txt &
echo "running..."
while true; do
    sleep 1
    if grep -q -i "Finished initing" TEST.log.txt; then
        echo "tests passed!"
        exit 0
    fi

    if grep -q -i "Kernel panic" TEST.log.txt; then
        echo "tests failed!"
        exit 1
    fi
done
