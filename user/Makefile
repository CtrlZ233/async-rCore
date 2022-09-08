TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
APPS := $(wildcard $(APP_DIR)/*.rs)

OBJDUMP := rust-objdump --arch-name=riscv64


elf: $(APPS)
	@cargo build --release

build: elf

clean:
	@cargo clean

.PHONY: elf binary build clean
