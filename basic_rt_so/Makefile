# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
BASIC_RT_ELF := target/$(TARGET)/$(MODE)/basic_rt_so
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
BOOTLOADER := ../bootloader/rustsbi-qemu.bin
BASIC_RT_ENTRY := 0x80200000

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif


# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

build: env so

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

so:
	@cargo build $(MODE_ARG)

clean:
	@cargo clean

run: run-inner

	
run-inner: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(BASIC_RT_ELF),addr=$(BASIC_RT_ENTRY)

disasm: so
	@$(OBJDUMP) $(DISASM) $(BASIC_RT_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(BASIC_RT_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)


	
.PHONY: build env so clean disasm disasm-vim 
