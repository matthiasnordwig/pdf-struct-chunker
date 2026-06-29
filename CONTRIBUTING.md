# Contributing to pdf-struct-chunker

Thanks for your interest in contributing! Here's how to get started.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_FORK/pdf-struct-chunker.git
   cd pdf-struct-chunker
   ```
3. Create a feature branch:
   ```bash
   git checkout -b feature/my-improvement
   ```

## Development

### Prerequisites
- Rust 1.85+ (install via [rustup](https://rustup.rs))

### Build & Test
```bash
cargo build          # Build the project
cargo test           # Run all tests
cargo clippy         # Check for common mistakes
cargo fmt            # Format your code
```

### Before Submitting a PR
Please make sure all of the following pass:
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## What to Contribute

- **New pattern roles** — e.g., `heading_l2`, `footnote`, `table_header`
- **Better heuristics** — improved font-size detection, multi-column support
- **More test fixtures** — PDFs from different domains (medical, financial, technical)
- **Documentation** — typo fixes, better examples, translations
- **Bug reports** — open an issue with a sample PDF if possible

## Code Style

- Follow `cargo fmt` defaults
- Keep functions focused and well-documented
- Add tests for new functionality
- Write doc comments (`///`) for all public items

## Reporting Bugs

Open a [GitHub Issue](https://github.com/matthiasnordwig/pdf-struct-chunker/issues) with:
- A description of the problem
- The PDF that caused it (if possible)
- The profile JSON you used (if any)
- Expected vs. actual output

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
