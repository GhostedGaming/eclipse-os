# Eclipse Framebuffer

A lightweight, `no_std` framebuffer text rendering library designed for bare-metal Rust projects using the Limine bootloader. Features scrolling text support and a familiar `println!` macro interface.

## Features

- **Zero dependencies** - Pure `no_std` Rust implementation
- **Direct framebuffer access** - Render text directly to video memory
- **Automatic scrolling** - Built-in scrolling text renderer
- **Familiar API** - `println!` macro just like std
- **PSF font support** - Embed PSF/PSF2 fonts at compile-time using `include_bytes!`
- **Limine bootloader integration** - Seamless setup with Limine's framebuffer protocol
- **Bare-metal ready** - Perfect for OS development and bootloader environments
- **Multi-architecture** - Supports x86_64, aarch64, riscv64, and loongarch64

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
eclipse-framebuffer = "0.1.0"
```

## Quick Start

```rust
#![no_std]
#![no_main]

use limine::request::FramebufferRequest;
use eclipse_framebuffer::{ScrollingTextRenderer, println};

static FONT: &[u8] = include_bytes!("../fonts/Mik_8x16.psf");

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    let framebuffer_response = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("No framebuffer");
    
    let framebuffer = framebuffer_response
        .framebuffers()
        .next()
        .expect("No framebuffer available");
    
    // Initialize the scrolling text renderer
    ScrollingTextRenderer::init(
        framebuffer.addr(),
        framebuffer.width() as usize,
        framebuffer.height() as usize,
        framebuffer.pitch() as usize,
        framebuffer.bpp() as usize,
        FONT,
    );
    
    // Now you can use println! just like in std
    println!("Hello from Eclipse Framebuffer!");
    println!("Framebuffer: {}x{}", framebuffer.width(), framebuffer.height());
    
    loop {}
}
```

## Usage

### ScrollingTextRenderer

The `ScrollingTextRenderer` provides automatic text scrolling when the screen fills up:

```rust
use eclipse_framebuffer::{ScrollingTextRenderer, println};

// After initialization...
println!("Line 1");
println!("Line 2");
println!("Line 3");
// Automatically scrolls when screen is full!
```

### Formatting Support

The `println!` macro supports all standard Rust formatting:

```rust
println!("Number: {}", 42);
println!("Hex: 0x{:X}", 0xDEADBEEF);
println!("Debug: {:?}", some_struct);
```

## Font Format

Eclipse Framebuffer supports **PSF (PC Screen Font)** and **PSF2** formats. These are simple bitmap font formats commonly used in console applications.

### Including Fonts

Place your `.psf` font files in your project and include them at compile time:

```rust
static FONT_8X16: &[u8] = include_bytes!("../fonts/Mik_8x16.psf");
static FONT_8X8: &[u8] = include_bytes!("../fonts/default_8x8.psf");
```

### Where to Find PSF Fonts

- Linux console fonts (usually in `/usr/share/consolefonts/`)
- [kbd project](https://github.com/legionus/kbd) - Large collection of console fonts
- Custom PSF fonts can be created with tools like `psftools`

## Architecture Support

Eclipse Framebuffer works across multiple architectures:

- **x86_64** - Intel/AMD 64-bit
- **aarch64** - ARM 64-bit
- **riscv64** - RISC-V 64-bit  
- **loongarch64** - LoongArch 64-bit

## API Overview

### Core Types

- **`ScrollingTextRenderer`** - Main renderer with automatic scrolling
- **`println!`** - Macro for formatted text output (like `std::println!`)

### Initialization

```rust
ScrollingTextRenderer::init(
    addr: *mut u8,          // Framebuffer address
    width: usize,           // Screen width in pixels
    height: usize,          // Screen height in pixels
    pitch: usize,           // Bytes per scanline
    bpp: usize,             // Bits per pixel
    font: &'static [u8],    // PSF font data
);
```

## Integration with Limine

Eclipse Framebuffer is designed to work seamlessly with the [Limine bootloader](https://github.com/limine-bootloader/limine):

1. Set up Limine framebuffer request
2. Get framebuffer info from Limine response
3. Initialize `ScrollingTextRenderer` with framebuffer parameters
4. Use `println!` throughout your kernel

See the Quick Start example above for complete integration code.

## Use Cases

Eclipse Framebuffer is ideal for:

-  **Operating system development** - Early boot text output and logging
-  **Embedded systems** - Direct framebuffer control without OS overhead
-  **Bare-metal applications** - Low-level graphics in `no_std` environments
-  **Kernel debugging** - Visual output before full driver initialization

## Performance

- Zero-copy rendering directly to framebuffer
- Efficient scrolling implementation
- Minimal memory overhead
- No heap allocation required

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [EclipseOS](https://github.com/GhostedGaming/eclipse-os) - Operating system using Eclipse Framebuffer
- [Limine](https://github.com/limine-bootloader/limine) - Modern bootloader
- [limine-rs](https://github.com/jasondyoungberg/limine-rs) - Rust bindings for Limine

## Acknowledgments

Built for the Rust bare-metal development community. Special thanks to the Limine bootloader project for providing excellent tooling for OS development.

---

**Note**: This is a `no_std` library designed for bare-metal environments. It requires a bootloader (like Limine) that provides framebuffer access.