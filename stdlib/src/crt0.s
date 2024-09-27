section .text
global _start

extern __stdlib__init__
extern main
extern exit

_start:
  mov rbp, 0
  push rbp
  push rbp

  push rdi
  push rsi

  call __stdlib__init__

  pop rsi
  pop rdi
  call main
  call exit
  ; unreachable
  hlt
