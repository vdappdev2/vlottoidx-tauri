# Changelog

All notable changes to vLottoIDX will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha] - 2025-08-23

### Added
- Initial alpha release of vLottoIDX desktop application
- Multi-chain support (VRSC, VRSCTEST, VARRR, VDEX, CHIPS, ...)
- Automatic chain discovery and RPC connection
- Secure credential storage using OS keychain
- Marketplace functionality:
  - Browse available offers
  - Create new offers (identity and currency)
  - Take existing offers
  - Close your open offers
- Identity management:
  - Create new identities
  - Update identity information
  - Set identity timelocks
  - Revoke and recover identities
- Currency operations:
  - View currency balances
  - Send currencies
  - Convert between currencies
  - Create new currencies (multiple types)
- DeFi operations support (11 operation types)
- Wallet overview with address management
- Dark/light theme toggle
- Cross-platform support (macOS, Windows, Linux)

### Fixed
- Critical HTTP 400 error on Intel macOS production builds
  - Root cause: Rust 1.88.0 compiler bug causing malformed HTTP requests
  - Solution: Applied opt-level=1 workaround in Cargo.toml
  - Details: [Rust Issue #144163](https://github.com/rust-lang/rust/issues/144163)

### Technical
- Built with Tauri 2.6.2 + Svelte 5 + TypeScript
- Rust backend for all RPC operations
- Frontend never handles credentials directly
- Uses verus-typescript-primitives for type safety
- Modular architecture for maintainability

### Known Issues
- Application binaries are unsigned (not certain if code signing coming in future release)
- Users need to bypass OS security warnings on first launch

### Notes
This is an alpha release for testing and feedback. Please report issues and user feedback in Verus Discord builder's forum thread for vLottoIDX; or on GitHub at https://github.com/vdappdev2/vlottoidx-tauri 