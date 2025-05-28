# Variables
OVMF_CODE = /usr/share/edk2/x64/OVMF_CODE.4m.fd
OVMF_VARS = /usr/share/edk2/x64/OVMF_VARS.4m.fd
FAT_IMG = fat.img
ISO_FILE = eclipse-os.iso
# Default to debug unless RELEASE=1 is set
BOOTLOADER_BUILD_DIR := $(if $(RELEASE),release,debug)
BOOTLOADER_PATH = $(CURDIR)/target/x86_64-unknown-uefi/$(BOOTLOADER_BUILD_DIR)/eclipse_os.efi
ESP_DIR = esp/efi/boot

.PHONY: run clean build-kernel build-bootloader check-artifacts esp fat iso qemu rust-clean

run: iso
	# Run with QEMU
	$(MAKE) qemu

build-bootloader:
	cargo build $(if $(RELEASE),--release,) --target x86_64-unknown-uefi

check-artifacts: build-kernel build-bootloader
	@if [ ! -f $(BOOTLOADER_PATH) ]; then echo "Error: eclipse_os.efi not found!"; exit 1; fi

esp: check-artifacts
	mkdir -p $(ESP_DIR)
	cp $(BOOTLOADER_PATH) $(ESP_DIR)/bootx64.efi

fat: esp
	dd if=/dev/zero of=$(FAT_IMG) bs=1M count=33
	mformat -i $(FAT_IMG) -F ::
	mmd -i $(FAT_IMG) ::/EFI
	mmd -i $(FAT_IMG) ::/EFI/BOOT
	mcopy -i $(FAT_IMG) $(ESP_DIR)/bootx64.efi ::/EFI/BOOT

iso: fat
	mkdir -p iso
	cp $(FAT_IMG) iso/
	xorriso -as mkisofs -R -f -e $(FAT_IMG) -no-emul-boot -o $(ISO_FILE) iso

qemu: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-device qemu-xhci -device usb-kbd -audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 --serial stdio -M q35 --no-reboot

clean:
# Delete: the ISO, FAT image, ESP directory, and the build artifacts
	rm -rf iso $(FAT_IMG) $(ESP_DIR) $(ISO_FILE)

rust-clean:
	cargo clean