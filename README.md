# 🌌 Eclipse OS

![Rust](https://img.shields.io/badge/Rust-🦀-orange?style=flat-square)
![License](https://img.shields.io/badge/License-Apache_2.0-blue?style=flat-square)
![Contributions](https://img.shields.io/badge/Contributions-Welcome-brightgreen?style=flat-square)

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<table>
  <tr>
    <td align="center"><a href="https://github.com/GhostedGaming"><img src="https://avatars.githubusercontent.com/u/180805056?v=4" width="100px;" alt=""/><br /><sub><b>GhostedGaming</b></sub></a><br />💻 🖋️</td>
  </tr>
</table>

Eclipse OS is a **lightweight** operating system written in **Rust**, designed for performance and simplicity. Explore the world of operating system development with this beginner-friendly project! 🚀

---

## ✨ Features

- 🦀 **Rust-based**: Built with the powerful and safe Rust programming language.
- 💾 **Planned File System Support**: Ext4 and NTFS support on the roadmap.
- 🌍 **Cross-platform Compatibility**: Build and test on major platforms.
- ⚙️ **Customizable**: Easily extend and add new features.

---

## 📦 Installation

### Prerequisites

- 🦀 **Rust**: Install Rust from [rustup.rs](https://rustup.rs/).
- 💻 **QEMU**: Virtual machine emulator for testing.

### Install on Windows

```bash
winget install --id Rustlang.Rustup
winget install --id SoftwareFreedomConservancy.QEMU -e
```

### Install on Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install qemu-system
```

---

## 🛠️ Building Eclipse OS

To build Eclipse OS, ensure you have the **nightly Rust toolchain** installed:

```bash
rustup install nightly
rustup override set nightly
```

Then, build the project:

```bash
cargo build
```

To create a bootable disk image:

```bash
cargo install bootimage
cargo bootimage
```

---

## 🧪 Testing

Run the unit and integration tests with:

```bash
cargo test
```

---

## 🤝 Contributing

We **welcome contributions** from developers of all skill levels! 🛠️

1. **Fork** the repository.
2. **Clone** your fork:
   ```bash
   git clone https://github.com/<your-username>/eclipse-os.git
   ```
3. Create a **branch** for your feature or fix:
   ```bash
   git checkout -b my-feature-branch
   ```
4. Push your changes and open a **pull request**.

For detailed guidelines, check out the [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 💬 Community

Join the conversation, ask questions, and share ideas:

- 🗨️ **[GitHub Discussions](https://github.com/GhostedGaming/eclipse-os/discussions)**
- 🎮 **[Discord Server](https://discord.gg/your-discord-link)**

---

## 📜 License

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for more details.

---

## 🛡️ Acknowledgments

Special thanks to the amazing **Rust community** and all contributors who make open-source projects possible! ❤️
