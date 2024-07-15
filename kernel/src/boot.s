# Inspired by Stephen Marz from 8 February 2019
.option norvc
.section .data

.section .text.init
.global _start
_start:
    # Any hardware threads (hart) that are not bootstrapping
    # need to wait for an IPI
    csrr t0, mhartid # read our hart identifier into the register t0 (control status register read)
    bnez t0, 3f # Check if it is zero, if not, send the hart in a busy loop
    # SATP should be zero, but let's make sure
    csrw satp, zero # Set Supervisor Address Translation and Protection to 0, because we don't have MMU for now

.option push
.option norelax
    # The reset vector of some boards will load "mhartid" into that hart's a0 register. 
    # However, some boards may choose not to do this, so we ensure it
    la gp, _global_pointer
.option pop

# Clearing the BSS (global uninitialized variable)
# We need to ensure they are all set to 0
    la a0, _bss_start # Load address bss_start
    la a1, _bss_end
    # Should never happend (_bss_start>bss_end)
    bgeu a0, a1, 2f # if a0 >= a1 {{ goto 2 }} 
1:
    sd zero, (a0) # Store double-word (8 bytes / 64bit) 0 at address in a0
    addi a0, a0, 8 # Offset a0 "ptr" of 8 bytes (point to next address to set to 0)
    bltu a0, a1, 1b # if current_ptr < end {{ repeat }}
2:  
    la sp, _stack_end # Load stack ptr, grows downwards, so we want a bit of space
    # Machine mode will give us access to all of the instructions and registers, we should already be in this state, but we don't know
    li		t0, (0b11 << 11) # | (1 << 7) | (1 << 3) # | (1 << 5)
    csrw	mstatus, t0
    csrw	mie, zero # Don't want interrupts when we haven't set them up yet
    la		t1, kmain # Load address of our main function (see src/main.rs)
    csrw	mepc, t1 # Set Machine Exception Program Counter
    la		ra, 3f
    # We use mret here so that the mstatus register is properly updated.
    mret

3:
    wfi
    j   3b
