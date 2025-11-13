require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "reactnative-lightening-wallet-rs"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => "12.0" }
  s.source       = { :git => "https://github.com/elementfm/reactnative-lightening-wallet-rs.git", :tag => "#{s.version}" }

  s.source_files = "ios/**/*.{h,m,mm,swift}"
  s.swift_version = "5.0"

  # Vendored frameworks (will contain the compiled Rust library)
  s.vendored_libraries = "ios/lib/libreactnative_lightening_wallet.a"

  # React Native dependencies
  s.dependency "React-Core"

  # Build settings
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'SWIFT_OBJC_BRIDGING_HEADER' => '$(PODS_TARGET_SRCROOT)/ios/LighteningWallet-Bridging-Header.h',
    'LIBRARY_SEARCH_PATHS' => '$(PODS_TARGET_SRCROOT)/ios/lib'
  }

  # Prepare command to build Rust library
  s.prepare_command = <<-CMD
    set -e

    # Navigate to Rust directory
    cd rust

    # Build for iOS simulators (x86_64 and arm64)
    cargo build --release --target x86_64-apple-ios
    cargo build --release --target aarch64-apple-ios-sim

    # Build for iOS devices (arm64)
    cargo build --release --target aarch64-apple-ios

    # Create output directory
    mkdir -p ../ios/lib

    # Create universal library for simulator
    lipo -create \
      target/x86_64-apple-ios/release/libreactnative_lightening_wallet.a \
      target/aarch64-apple-ios-sim/release/libreactnative_lightening_wallet.a \
      -output ../ios/lib/libreactnative_lightening_wallet_sim.a

    # Copy device library
    cp target/aarch64-apple-ios/release/libreactnative_lightening_wallet.a \
      ../ios/lib/libreactnative_lightening_wallet_device.a

    # Create XCFramework
    xcodebuild -create-xcframework \
      -library ../ios/lib/libreactnative_lightening_wallet_sim.a \
      -library ../ios/lib/libreactnative_lightening_wallet_device.a \
      -output ../ios/LighteningWallet.xcframework

    # For pod install, use the simulator library by default
    cp ../ios/lib/libreactnative_lightening_wallet_sim.a \
      ../ios/lib/libreactnative_lightening_wallet.a
  CMD
end
