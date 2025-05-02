# Contributing to Eclipse OS

First off, thank you for considering contributing to Eclipse OS! 🎉 Your contributions, whether they're bug reports, feature suggestions, or code, are greatly appreciated.

---

## 🛠️ How Can I Contribute?

### 1. Reporting Bugs 🐛
If you find a bug, we’d love to hear about it! Please open an issue and include:
- A clear, descriptive title (e.g., "Crash on boot after enabling X").
- Steps to reproduce the issue.
- The expected outcome vs. the actual outcome.
- Any relevant logs or screenshots.

You can open a new bug report [here](https://github.com/GhostedGaming/eclipse-os/issues/new).

---

### 2. Suggesting Enhancements 💡
Have an idea for a new feature? Great! Open an issue with the following:
- A descriptive title (e.g., "Add support for ext4 file system").
- The problem your suggestion solves.
- Any relevant examples or references to guide us.

You can suggest enhancements [here](https://github.com/GhostedGaming/eclipse-os/issues/new).

---

### 3. Submitting Code Contributions 🖋️

We welcome code contributions from developers of all levels! To get started:

#### Step 1: Fork the Repository
Click the **"Fork"** button at the top-right corner of this page to create your own copy of the project.

#### Step 2: Clone the Repository
Clone your fork to your local machine:
```bash
git clone https://github.com/<your-username>/eclipse-os.git
cd eclipse-os
```

#### Step 3: Create a Branch
Create a branch for your feature or fix:
```bash
git checkout -b my-feature-branch
```

#### Step 4: Make Changes
Make your changes, and ensure your code follows the existing coding style.

#### Step 5: Test Your Changes
Run the tests to ensure your changes don’t break anything:
```bash
cargo xtest
```

#### Step 6: Commit and Push
Commit your changes with a meaningful message:
```bash
git add .
git commit -m "Add feature X"
git push origin my-feature-branch
```

#### Step 7: Open a Pull Request
Go to your fork on GitHub and click the **"Pull Request"** button. Fill out the PR template and submit it. 🎉

---

## 📝 Code Style Guidelines
- Follow Rust's standard coding style. Use `rustfmt` to format your code:
  ```bash
  rustfmt src/*.rs
  ```
- Keep commits atomic and meaningful.
- Document your code where necessary for better readability.

---

## 🙌 Community Guidelines
- Be respectful and inclusive.
- Provide constructive feedback in code reviews.
- Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

---

## 📄 License
By contributing to Eclipse OS, you agree that your contributions will be licensed under the **Apache License 2.0**.

---

Thank you for contributing to Eclipse OS! 🦀✨
