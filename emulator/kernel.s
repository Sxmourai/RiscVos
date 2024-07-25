.global _start

_start:
    li      s1, 0x10000000
    li      s2, 'h'
    sb      s2, 0(s1)

loop:
    j _start
