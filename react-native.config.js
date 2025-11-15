module.exports = {
  dependency: {
    platforms: {
      android: {
        sourceDir: './android',
        packageImportPath: 'import com.reactnativelighteningwallet.LighteningWalletPackage;',
        packageInstance: 'new LighteningWalletPackage()',
      },
      ios: {
        project: './ios/LighteningWallet.xcodeproj',
      },
    },
  },
};
