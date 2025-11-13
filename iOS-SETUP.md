# iOS Setup Guide

Complete guide for building and using the Lightning Wallet on iOS.

## Prerequisites

1. **Xcode** 12 or later
2. **Rust** with iOS toolchains
3. **CocoaPods**

## Install iOS Rust Targets

```bash
rustup target add aarch64-apple-ios         # iOS devices (ARM64)
rustup target add x86_64-apple-ios          # iOS simulator (Intel Mac)
rustup target add aarch64-apple-ios-sim     # iOS simulator (Apple Silicon Mac)
```

## Build the Library

### Option 1: Automated Build (Recommended)

```bash
cd ~/code/element.fm/reactnative-lightening-wallet-rs
npm run build:ios
```

This script will:
- Build for all iOS architectures
- Create a universal simulator library
- Create an XCFramework
- Set up the correct symlinks

### Option 2: Manual Build

```bash
cd ~/code/element.fm/reactnative-lightening-wallet-rs/rust

# Build for iOS device (ARM64)
cargo build --release --target aarch64-apple-ios

# Build for iOS simulator (Intel)
cargo build --release --target x86_64-apple-ios

# Build for iOS simulator (Apple Silicon)
cargo build --release --target aarch64-apple-ios-sim

# Create directories
mkdir -p ../ios/lib

# Create universal simulator library
lipo -create \
  target/x86_64-apple-ios/release/libreactnative_lightening_wallet.a \
  target/aarch64-apple-ios-sim/release/libreactnative_lightening_wallet.a \
  -output ../ios/lib/libreactnative_lightening_wallet_sim.a

# Copy device library
cp target/aarch64-apple-ios/release/libreactnative_lightening_wallet.a \
  ../ios/lib/libreactnative_lightening_wallet_device.a
```

## Integration into Your React Native App

### 1. Install the Package

```bash
cd /path/to/your/react-native/app
npm install ~/code/element.fm/reactnative-lightening-wallet-rs
```

### 2. Install iOS Dependencies

```bash
cd ios
pod install
cd ..
```

The Podspec's `prepare_command` will automatically build the Rust library during `pod install`. If you've already built it manually, this step will be skipped.

### 3. Open in Xcode

```bash
cd ios
open YourApp.xcworkspace  # NOT .xcodeproj
```

### 4. Build Settings

The Podspec automatically configures:
- Swift/Objective-C bridging header
- Library search paths
- Framework search paths

No manual Xcode configuration needed!

## File Structure

After building, you should have:

```
reactnative-lightening-wallet-rs/
├── ios/
│   ├── LighteningWallet.swift              # Swift bridging code
│   ├── LighteningWallet.m                  # Objective-C module definition
│   ├── LighteningWallet-Bridging-Header.h  # C FFI declarations
│   └── lib/
│       ├── libreactnative_lightening_wallet.a           # Symlink (current arch)
│       ├── libreactnative_lightening_wallet_sim.a       # Simulator (universal)
│       └── libreactnative_lightening_wallet_device.a    # Device (ARM64)
└── rust/
    └── target/
        ├── aarch64-apple-ios/release/
        ├── x86_64-apple-ios/release/
        └── aarch64-apple-ios-sim/release/
```

## Testing

### Test on Simulator

```bash
# Make sure you've built for simulator
npm run build:ios

# Run on simulator
cd ios
xcodebuild -workspace YourApp.xcworkspace -scheme YourApp -sdk iphonesimulator
```

Or use Xcode:
1. Select an iOS Simulator (any will work with universal binary)
2. Click Run (⌘R)

### Test on Device

```bash
# Make sure you've built for device
npm run build:ios

# Connect your iPhone/iPad
# Select it in Xcode
# Click Run (⌘R)
```

## Troubleshooting

### Error: "library not found"

**Problem:** Xcode can't find the Rust library.

**Solution:**
```bash
# Rebuild the library
cd ~/code/element.fm/reactnative-lightening-wallet-rs
npm run build:ios

# Reinstall pods
cd /your/app/ios
pod deintegrate
pod install
```

### Error: "Undefined symbols for architecture"

**Problem:** Built for wrong architecture.

**Solution:**
```bash
# For simulator, make sure you have both:
ls -la ~/code/element.fm/reactnative-lightening-wallet-rs/ios/lib/

# You should see:
# libreactnative_lightening_wallet_sim.a

# If missing, rebuild:
npm run build:ios
```

### Error: "Module 'LighteningWallet' not found"

**Problem:** Swift module not properly configured.

**Solution:**
1. Clean build folder: Product → Clean Build Folder (⌘⇧K)
2. Delete `Pods/` and `Podfile.lock`
3. Run `pod install` again
4. Rebuild

### Build Takes Forever on First Run

This is normal! The first build:
- Downloads all Rust dependencies
- Compiles for 3 architectures
- Creates universal libraries

Subsequent builds are much faster thanks to incremental compilation.

**Tip:** Use `cargo build` (without `--release`) during development for faster builds:

```bash
cd rust
cargo build --target aarch64-apple-ios-sim
```

### Simulator vs Device Library

The library automatically switches based on build target:
- **Simulator builds** use `libreactnative_lightening_wallet_sim.a` (Intel + Apple Silicon)
- **Device builds** use `libreactnative_lightening_wallet_device.a` (ARM64 only)

The symlink `libreactnative_lightening_wallet.a` points to simulator by default for development.

## Performance Notes

### Build Time

| Target | First Build | Incremental |
|--------|------------|-------------|
| Simulator (Intel) | ~5-8 min | ~30 sec |
| Simulator (ARM) | ~5-8 min | ~30 sec |
| Device (ARM64) | ~5-8 min | ~30 sec |
| All (npm script) | ~15-20 min | ~1-2 min |

### Binary Size

| Library | Size |
|---------|------|
| Simulator (universal) | ~35 MB |
| Device (ARM64) | ~18 MB |
| XCFramework | ~50 MB |

**Note:** Release builds use LTO and optimization, resulting in smaller binaries in production.

## Distribution

### For Development

Ship the XCFramework:
```
ios/LighteningWallet.xcframework/
```

### For Production

The library will be statically linked into your app binary. No separate framework distribution needed.

Final app size increase: ~15-20 MB (after compression and dead code elimination).

## Next Steps

- See [QUICKSTART.md](./QUICKSTART.md) for usage examples
- Check [README.md](./README.md) for full API documentation
- Review [examples/](./examples/) for complete code samples

## CocoaPods Advanced Configuration

### Custom Build Configuration

If you need custom build settings, create a `Podfile.local` in your iOS directory:

```ruby
# Podfile.local
post_install do |installer|
  installer.pods_project.targets.each do |target|
    if target.name == 'reactnative-lightening-wallet-rs'
      target.build_configurations.each do |config|
        # Custom settings here
        config.build_settings['ENABLE_BITCODE'] = 'NO'
      end
    end
  end
end
```

### Skip Automatic Rust Build

If you're building Rust manually, you can skip the Podspec's `prepare_command`:

```bash
# Set environment variable
export SKIP_RUST_BUILD=1
pod install
```

## Support

Having issues with iOS builds? Check:

1. Rust version: `rustc --version` (should be 1.70+)
2. Xcode version: `xcodebuild -version` (should be 12+)
3. Targets installed: `rustup target list | grep apple-ios`
4. Build logs: `~/code/element.fm/reactnative-lightening-wallet-rs/rust/target/`

For help, open an issue with:
- Your Xcode version
- Your Rust version
- Full build output
- Platform (M1/M2/Intel Mac)
