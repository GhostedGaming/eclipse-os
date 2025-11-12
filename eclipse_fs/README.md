# eclipse_fs

A no_std inode-based filesystem for embedded and bare-metal environments. Supports both IDE and AHCI storage controllers.

## Features

- Inode-based file system architecture
- Block bitmap allocation
- File creation, reading, and deletion
- Directory support with file lookup

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
eclipse_fs = "0.1.0"
```

## Quick Start

```rust
use eclipse_fs::{SuperBlock, BlockBitmap, InodeManager, write_eclipse_fs};
use eclipse_fs::file_ops::{create_file, read_file, delete_file};
use eclipse_fs::directory::DirectoryManager;

// Initialize filesystem
write_eclipse_fs(0);

let superblock = SuperBlock::read_super_block(0)?;
let bitmap = BlockBitmap::from_disk(0, &superblock)?;
let mut inode_manager = InodeManager::new(0, superblock, bitmap)?;

// Create a file
let data = b"Hello, EclipseOS!";
let inode_idx = create_file(&mut inode_manager, data)?;

// Read it back
let file_data = read_file(&inode_manager, inode_idx)?;

// Create a directory
let dir_inode = DirectoryManager::create_directory(&mut inode_manager)?;
DirectoryManager::add_entry(&mut inode_manager, dir_inode, b"myfile.txt", inode_idx)?;
```

## Components

- **Superblock**: Filesystem metadata and configuration
- **Inodes**: File metadata with direct and indirect block pointers
- **Bitmap**: Free/allocated block tracking
- **File Operations**: Create, read, delete files
- **Directories**: File organization and lookup
- **Block I/O**: Storage driver abstraction layer

## Limitations

- No subdirectories yet
- 256-byte maximum filename length
- No file permissions or timestamps
- Linear directory search

## Needed features

- Modular functions so people can use their own drivers
- Some other FS stuff

## License

MIT
