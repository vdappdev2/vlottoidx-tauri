# Contributing to vLottoIDX

Thank you for your interest in contributing to vLottoIDX! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Be respectful and constructive in all interactions. We're building something together.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

**To report a bug:**
1. Use the GitHub Issues page
2. Include a clear title and description
3. Provide:
   - Your OS and version
   - Verus daemon version
   - Steps to reproduce the issue
   - Expected vs actual behavior
   - Error messages or screenshots if applicable
   - screenshots are very helpful

### Suggesting Features

1. Check if the feature has already been suggested
2. Open a new issue with the "enhancement" label
3. Clearly describe the feature and its use case
4. Explain how it benefits vLottoIDX users
5. Include screenshot if applicable

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/vlottoidx-tauri.git
cd vlottoidx-tauri

# Install dependencies
npm install

# Start development
npm run tauri:dev
```

### Pull Request Process

1. **Fork the repository** and create your branch from `main`
2. **Follow the architecture**:
   - ALL RPC logic must be in the Rust backend (`src-tauri/`)
   - Frontend (`src/`) should never handle RPC credentials
   - Use the existing modular structure for new features
3. **Before submitting**:
   ```bash
   # Run all quality checks
   npm run lint
   npm run type-check
   npm run format:check
   npm run rust:check
   npm run rust:clippy
   ```
4. **Commit messages** should be clear and descriptive
5. **Update documentation** if you change functionality
6. **Test thoroughly** with a running Verus daemon

### Code Style

#### TypeScript/Svelte
- Use TypeScript for all new code
- Follow the existing Svelte 5 patterns
- Use Tailwind CSS for styling
- Maintain type safety with `verus-typescript-primitives`

#### Rust
- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Handle errors properly with descriptive messages

### Architecture Guidelines

1. **Security First**
   - Never log sensitive data (passwords, private keys)
   - Use OS keychain for credential storage
   - Validate all inputs in Rust before RPC calls

2. **Separation of Concerns**
   - Rust backend: RPC communication, credential management, security
   - Svelte frontend: UI, user interaction, display logic
   - Clear module boundaries for marketplace, identity, wallet features

3. **Async/Await**
   - Use async patterns throughout for non-blocking operations
   - Proper error handling for all async operations

### Testing

- Test with both VRSC mainnet and VRSCTEST testnet
- Verify offline functionality (app should handle daemon being down)
- Test error scenarios (wrong password, daemon not running, etc.)

### Questions?

Feel free to open an issue for any questions about contributing. We appreciate your help in making vLottoIDX better!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.