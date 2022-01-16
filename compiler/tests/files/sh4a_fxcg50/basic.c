int main() {
	asm("
			mov.l R0, 0x0EAB
			mov.l R4, 0x80020070
			jsr R4
			nop
			");

	return 0;
}
