.global _start

_start:
    li      s1, 0x10000000
    li      s2, 'j'
    j loop
loop:

    sb      s2, 0(s1)

    j _start
