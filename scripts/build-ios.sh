#!/bin/bash

set -e

echo "Building Rust library for iOS..."

cd "$(dirname "$0")/../rust"

# Install iOS targets if not already installed
echo "Installing Rust iOS targets..."
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim

# Build for iOS simulators (Intel)
echo "Building for iOS simulator (x86_64)..."
cargo build --release --target x86_64-apple-ios

# Build for iOS simulators (Apple Silicon)
echo "Building for iOS simulator (aarch64)..."
cargo build --release --target aarch64-apple-ios-sim

# Build for iOS devices
echo "Building for iOS device (aarch64)..."
cargo build --release --target aarch64-apple-ios

# Create output directory
mkdir -p ../ios/lib

# Create universal library for simulator
echo "Creating universal simulator library..."
lipo -create \
  target/x86_64-apple-ios/release/libreactnative_lightening_wallet.a \
  target/aarch64-apple-ios-sim/release/libreactnative_lightening_wallet.a \
  -output ../ios/lib/libreactnative_lightening_wallet_sim.a

# Copy device library
echo "Copying device library..."
cp target/aarch64-apple-ios/release/libreactnative_lightening_wallet.a \
  ../ios/lib/libreactnative_lightening_wallet_device.a

# Create XCFramework (optional, for better distribution)
echo "Creating XCFramework..."
if [ -d "../ios/LighteningWallet.xcframework" ]; then
  rm -rf ../ios/LighteningWallet.xcframework
fi

xcodebuild -create-xcframework \
  -library ../ios/lib/libreactnative_lightening_wallet_sim.a \
  -library ../ios/lib/libreactnative_lightening_wallet_device.a \
  -output ../ios/LighteningWallet.xcframework

# For development, create a symlink to simulator library
echo "Creating development library symlink..."
ln -sf libreactnative_lightening_wallet_sim.a ../ios/lib/libreactnative_lightening_wallet.a

echo "✅ iOS build complete!"
echo ""
echo "Output files:"
echo "  - ios/lib/libreactnative_lightening_wallet_sim.a (Simulator)"
echo "  - ios/lib/libreactnative_lightening_wallet_device.a (Device)"
echo "  - ios/LighteningWallet.xcframework (Universal)"
echo ""
echo "To use in Xcode:"
echo "  1. Run 'pod install' in your iOS project"
echo "  2. Open the .xcworkspace file"
echo "  3. Build and run"
