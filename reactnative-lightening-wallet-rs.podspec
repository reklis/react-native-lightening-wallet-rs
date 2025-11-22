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
    'DEFINES_MODULE' => 'NO',
    'SWIFT_OBJC_BRIDGING_HEADER' => '$(PODS_TARGET_SRCROOT)/ios/LighteningWallet-Bridging-Header.h',
    'LIBRARY_SEARCH_PATHS' => '$(PODS_TARGET_SRCROOT)/ios/lib'
  }

  # Note: Pre-built iOS libraries are included in the npm package.
  # The prepare_command has been removed to avoid requiring Rust toolchain during pod install.
  # To rebuild iOS libraries, run: npm run build:ios
end
