/* Stolen from https://wiki.osdev.org/RISC-V_Bare_Bones and https://github.com/moratorium08/osmium/blob/master/kernel/linker.ld	 */

OUTPUT_ARCH("riscv")

ENTRY(start);
 
. = 0x80000000; /*0x00050000*/
 
SECTIONS {
	/* Include entry point at start of binary */
	.text : ALIGN(4K) {
		*(.init);
		*(.text);
	}
	.bss : ALIGN(4K) {
		PROVIDE(bss_start = .);
		*(.bss);
		. += 4096;
		PROVIDE(stack_top = .);
		. += 4096;
		PROVIDE(global_pointer = .);
		PROVIDE(bss_end = .);
	}
	.rodata : ALIGN(4K) {
		*(.rodata);
	}
	.data : ALIGN(4K) {
		*(.data);
	}
}