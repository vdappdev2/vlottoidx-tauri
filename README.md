# vLottoIDX

Desktop application for Verus blockchain with vLotto drawing viewer

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## About

vLottoIDX provides a desktop GUI for Verus RPC daemon features with integrated vLotto drawing viewer, offering secure and performant access to the Verus PBaaS network and vLotto on-chain drawing system through the VerusIDX interface.

## Features

- **vLotto Drawing Viewer**: Third-party viewer for vLotto on-chain drawing system with cryptographic verification
- **Charts**: Tradingview Lightweight Charts implementation
- **Marketplace Operations**: Browse, make, take, and close offers
- **Identity Management**: Create, update, revoke, recover, and timelock identities
- **Currency Operations**: Send, convert, and create currencies
- **DeFi Operations**: Access to 11 different DeFi operation types
- **Multi-chain Support**: VRSC, VRSCTEST, VARRR, VDEX, CHIPS
- **Secure Credential Storage**: OS keychain integration for RPC credentials

## Tech Stack

- **Frontend**: Svelte 5 + TypeScript + Tailwind CSS
- **Backend**: Rust + Tauri framework
- **Build System**: Vite
- **Platform**: Cross-platform desktop (macOS, Windows, Linux)

## Requirements

- **Verus Daemon**: Local Verus node required for RPC operations
- **System**: macOS 10.15+, Windows 10+, or Linux (Ubuntu 18.04+)

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/vdappdev2/vlottoidx-tauri/releases) page.

#### macOS Installation
If you see "vLottoIDX cannot be opened because the developer cannot be verified":
1. Right-click the vLottoIDX app and select "Open"
2. Click "Open" in the dialog that appears
3. The app will now run and be saved as an exception

Alternatively, from Terminal:
```bash
xattr -cr /Applications/vLottoIDX.app
```

#### Windows Installation
If Windows Defender blocks the app:
1. Click "More info" when SmartScreen appears
2. Click "Run anyway"

#### Linux Installation
For AppImage:
```bash
chmod +x vlottoidx-*.AppImage
./vlottoidx-*.AppImage
```

For .deb package:
```bash
sudo dpkg -i vlottoidx_*.deb
```

## Building from Source

### Prerequisites

- Rust 1.88.0+ (latest stable)
- Node.js 18+ and npm
- Running Verus daemon with RPC enabled

### Installation

```bash
# Clone the repository
git clone https://github.com/vdappdev2/vlottoidx-tauri.git
cd vlottoidx-tauri

# Install dependencies
npm install

# Build for production
npm run tauri:build
```

### Development

```bash
# Start development server
npm run tauri:dev
```

### Known Issues

#### Intel macOS HTTP 400 Error
If you encounter HTTP 400 errors on Intel macOS with production builds, this is due to a [Rust 1.88.0 compiler bug](https://github.com/rust-lang/rust/issues/144163). The workaround is already applied in `src-tauri/Cargo.toml` with `opt-level = 1`.

### Available Scripts

```bash
# Development
npm run dev              # Frontend dev server
npm run tauri:dev        # Full Tauri dev environment

# Building
npm run build           # Build frontend
npm run tauri:build     # Build desktop application

# Code Quality
npm run lint            # ESLint
npm run type-check      # TypeScript checking
npm run format          # Prettier formatting
npm run rust:check      # Cargo check
npm run rust:clippy     # Cargo clippy
```

## Usage

1. **Launch vLottoIDX**: Start the application
2. **RPC Connection**: App automatically discovers and connects to local Verus daemon
3. **Identity Selection**: Choose or create a sub-identity for marketplace access
4. **Explore Features**: Access vLotto viewer, marketplace, manage identities, perform DeFi operations

## Architecture

vLottoIDX follows a security-first design:

- **Backend Security**: All RPC operations handled in Rust backend
- **Credential Protection**: Never exposes RPC credentials to frontend
- **Modular Design**: Separate modules for marketplace, identity, wallet, and DeFi operations
- **Type Safety**: Full TypeScript integration with `verus-typescript-primitives`

## Contributing

### Development Guidelines

1. **Security**: Never log sensitive data or expose RPC credentials
2. **Code Style**: Use provided linting and formatting tools
3. **Testing**: Test RPC error scenarios and offline functionality
4. **Documentation**: Update relevant docs for any feature changes

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Run linting and type checking
4. Test with local Verus daemon
5. Submit pull request with clear description

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Attribution

TradingView Lightweight Charts™
Copyright (с) 2025 TradingView, Inc. https://www.tradingview.com/

## Links

- [Verus Coin](https://verus.io)
- [Verus PBaaS Documentation](https://wiki.verus.io)
- [Tauri Framework](https://tauri.app)
- [Svelte 5](https://svelte.dev)

## Acknowledgments

Built with the Verus PBaaS network and powered by the Tauri framework for secure, cross-platform desktop applications.