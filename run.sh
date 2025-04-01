#!/bin/bash

cargo bootimage

# Hard-coded paths
SOURCE_FILE="./target/x86_64-eclipse_os/debug/bootimage-eclipse_os.bin"
DESTINATION_DIR="./output"

# Create destination directory if it doesn't exist
mkdir -p "$DESTINATION_DIR"

# Copy the file
cp "$SOURCE_FILE" "$DESTINATION_DIR"

# Confirm copy was successful
if [ $? -eq 0 ]; then
    echo "Bootable image copied successfully to $DESTINATION_DIR"
else
    echo "Error copying bootable image"
    exit 1
fi

# Run QEMU with the bootable image
qemu-system-x86_64 -rtc base=localtime -drive format=raw,file="$DESTINATION_DIR/bootimage-eclipse_os.bin"