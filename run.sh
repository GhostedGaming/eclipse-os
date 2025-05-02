#!/bin/bash

cargo bootimage

# Hard-coded paths
SOURCE_FILE="./target/x86_64-eclipse_os/debug/bootimage-eclipse_os.bin"
DESTINATION_DIR="./output"
FAT32_DISK="$DESTINATION_DIR/fat32disk.img"

# Create destination directory if it doesn't exist
mkdir -p "$DESTINATION_DIR"

# Copy the bootable image
cp "$SOURCE_FILE" "$DESTINATION_DIR"

# Confirm copy was successful
if [ $? -eq 0 ]; then
    echo "Bootable image copied successfully to $DESTINATION_DIR"
else
    echo "Error copying bootable image"
    exit 1
fi

# Create a FAT32 virtual disk if it doesn't exist
#if [ ! -f "$FAT32_DISK" ]; then
#    echo "Creating new FAT32 virtual disk..."
    
    # Create an empty disk image (100MB)
#    qemu-img create -f raw "$FAT32_DISK" 100M
    
    # Format the disk with FAT32
    # On Linux:
#    mkfs.fat -F 32 "$FAT32_DISK"
#    
#    echo "FAT32 virtual disk created at $FAT32_DISK"
#fi

# Run QEMU with both the bootable image and the FAT32 disk
# Using more compatible settings
#qemu-system-x86_64 -rtc base=localtime \
#    -drive format=raw,file="$DESTINATION_DIR/bootimage-eclipse_os.bin" \
#    -drive format=raw,file="$FAT32_DISK",media=disk \
#    -vga std \
#    -m 128M
