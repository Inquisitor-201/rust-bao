rustup_target:=aarch64-unknown-none
toolchain_prefix:=aarch64-none-elf

target_dir:=target/$(rustup_target)/release
bao_elf:=$(target_dir)/rust-bao
bao_bin:=$(target_dir)/rust-bao.bin
bao_disasm:=$(target_dir)/rust-bao.asm
bao_elf_txt:=$(target_dir)/rust-bao.elf.txt

imgs_dir:=imgs
atf-fip:=$(imgs_dir)/flash.bin

OBJCOPY := rust-objcopy --binary-architecture=aarch64

qemu_cmd:=/home/clk/workspace/bao-demos/wrkdir/srcs/qemu/build/qemu-system-aarch64
qemu_flags:=-nographic\
		-M virt,secure=on,virtualization=on,gic-version=3 \
		-cpu cortex-a53 -smp 4 -m 4G\
		-bios $(atf-fip)\
		-device loader,file="$(bao_bin)",addr=0x50000000,force-raw=on\
		-device virtio-net-device,netdev=net0\
		-netdev user,id=net0,net=192.168.42.0/24,hostfwd=tcp:127.0.0.1:5555-:22\
		-device virtio-serial-device -chardev pty,id=serial3 -device virtconsole,chardev=serial3

build: env
	cargo build --release && make dump

dump:
	$(toolchain_prefix)-objdump -lS $(bao_elf) > $(bao_disasm)
	$(toolchain_prefix)-readelf -a $(bao_elf) > $(bao_elf_txt)

run: $(bao_bin)
	@$(qemu_cmd) $(qemu_flags)
	
gdb: $(bao_bin)
	@$(qemu_cmd) $(qemu_flags) -s -S

monitor:
	@gdb-multiarch -ex 'target remote localhost:1234'

env:
	rustup target add $(rustup_target)
	rustup component add llvm-tools-preview

clean:
	cargo clean

$(bao_bin): build
	@$(OBJCOPY) $(bao_elf) --strip-all -O binary $@

.PHONY: env build run gdb monitor clean dump