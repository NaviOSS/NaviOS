#!/bin/bash
make
git clone https://github.com/novnc/noVNC --depth 1
export qemu=qemu-system-i386
$qemu -kernel kernel.bin -vnc :0 &

echo "qemu starting..."
sleep 2
echo "done"

cd noVNC
echo "starting noVNC..."
./utils/novnc_proxy --vnc localhost:5900 &
sleep 2

xdg-open http://localhost:6080/vnc.html &
while true; do read -p "type exit to exit: " </dev/tty;  if [ "$REPLY" == "exit" ]; then break; fi; done
killall $qemu