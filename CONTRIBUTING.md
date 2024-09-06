# Contributing to AlterWare Launcher

We welcome contributions to the AlterWare Launcher! Here are some guidelines to follow:

> [!NOTE]
> Always run `cargo fmt` and `cargo clippy` to ensure your code is formatted correctly and passes lint checks.

### Prerequisites

- [Rust](https://rustup.rs/) - Install the latest stable version
- [Git](https://git-scm.com/) - For cloning the repository
- [Perl](https://www.perl.org/get.html) - [Linux only]  Required for OpenSSL

### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/mxve/alterware-launcher.git
   cd alterware-launcher
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Find the executable:
   The compiled binary will be located at `target/release/alterware-launcher(.exe)`

### Additional Notes

- Structs generally go in `src/structs.rs`
- Try to follow the existing coding style, make use of `cargo fmt` and `cargo clippy` to ensure consistency
- Use existing formatting, printing and helper functions when possible (see `src/misc.rs` and `src/extend.rs`)
- Unit tests go in `src/tests.rs`
- Make sure your code compiles on Windows and Unix targets
  - The GitHub Actions currently only run on Linux
    - You can use these to check for errors on unix platforms
    - You have to verify your code on Windows manually
- For debugging, you can use `cargo build` without the `--release` flag. The debug build will be slower but includes additional debugging information.
- To run tests, use `cargo test`

## License

By contributing to AlterWare Launcher, you agree that your contributions will be licensed under the [GPLv3 license](LICENSE).
