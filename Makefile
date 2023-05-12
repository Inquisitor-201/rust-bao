MODE?=debug
rustc_target:=aarch64-unknown-none
toolchain_prefix:=aarch64-none-elf

target_dir:=target/$(rustc_target)/$(MODE)
target_name:=rust-bao
bao_elf:=$(target_dir)/$(target_name)
bao_bin:=$(target_dir)/$(target_name).bin
bao_disasm:=$(target_dir)/$(target_name).asm
bao_elf_txt:=$(target_dir)/$(target_name).elf.txt

imgs_dir:=imgs
atf-fip:=$(imgs_dir)/flash.bin

OBJCOPY := rust-objcopy --binary-architecture=aarch64

qemu_cmd:=qemu-system-aarch64
qemu_flags:=-nographic\
		-M virt,secure=on,virtualization=on,gic-version=3 \
		-cpu cortex-a53 -smp 4 -m 4G\
		-bios $(atf-fip)\
		-device loader,file="$(bao_bin)",addr=0x50000000,force-raw=on\
		-device virtio-net-device,netdev=net0\
		-netdev user,id=net0,net=192.168.42.0/24,hostfwd=tcp:127.0.0.1:5555-:22\
		-device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3

ifeq ($(MODE), release)
    BUILD_CFG := --release
else
    BUILD_CFG := 
endif


build: env
	cargo build $(BUILD_CFG) && make dump

dump:
	$(toolchain_prefix)-objdump -lS $(bao_elf) > $(bao_disasm)
	$(toolchain_prefix)-readelf -a $(bao_elf) > $(bao_elf_txt)

run: $(bao_bin)
	@$(qemu_cmd) $(qemu_flags)

run-freertos:
	@$(qemu_cmd) -nographic\
		-M virt,secure=on,gic-version=3 \
		-cpu cortex-a53 -smp 4 -m 4G\
		-device loader,file="imgs/qemu-aarch64-virt/freertos.bin",addr=0x50000000,force-raw=on\
		-device virtio-net-device,netdev=net0\
		-netdev user,id=net0,net=192.168.42.0/24,hostfwd=tcp:127.0.0.1:5555-:22\
		-device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3
# -bios $(atf-fip)\


gdb: $(bao_bin)
	@$(qemu_cmd) $(qemu_flags) -s -S

monitor:
	@gdb-multiarch -ex 'target remote localhost:1234' \
		-ex 'file $(bao_elf)'

env:
	rustup target add $(rustc_target)
	rustup component add llvm-tools-preview

clean:
	cargo clean

show-features:
	rustc --print=target-features --target=$(rustc_target)

$(bao_bin): build
	@$(OBJCOPY) $(bao_elf) --strip-all -O binary $@

.PHONY: env build run gdb monitor clean dump show-features