ifeq ($(strip $(linux_image_path)),)
$(error linux_image_path is not defined)
endif

ifeq ($(strip $(linux_dts_path)),)
$(error linux_dts_path is not defined)
endif

ifeq ($(strip $(output_path)),)
$(error output_path is not defined)
endif

$(output_path): dtb
	aarch64-none-elf-gcc \
		-Wl,-build-id=none -nostdlib -T loader_aarch64.ld \
		-march=armv8-a\
		-D IMAGE=$(linux_image_path)\
		-D DTB=$(basename $(linux_dts_path)).dtb \
		aarch64.S \
		-o linux.elf \
	&& aarch64-none-elf-objcopy -S -O binary linux.elf $(output_path)

dtb:
	dtc -I dts -O dtb -o $(basename $(linux_dts_path)).dtb $(linux_dts_path)

.PHONY: clean
clean:
	rm -f $(basename $(linux_dts_path)).dtb $(output_path)
