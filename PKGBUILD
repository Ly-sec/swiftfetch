# Maintainer: Lysec

pkgname=swiftfetch
pkgver=0.1.0
pkgrel=1
pkgdesc="Fetch program written in rust"
arch=('x86_64')
url="https://github.com/Ly-sec/swiftfetch"
license=('MIT')
depends=('rust' 'cargo')
source=("https://github.com/Ly-sec/swiftfetch/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('c59a64b7d05d13fb6adf7d0a7cd3b76bedf40f714ac05eecaad7cf58e3766f04')

build() {
  # Extract the source directory and navigate into it
  cd "$srcdir/swiftfetch-$pkgver"
  
  # Build the project
  cargo build --release
}

package() {
  cd "$srcdir/swiftfetch-$pkgver"
  
  # Install the built binary
  install -Dm755 target/release/swiftfetch "$pkgdir/usr/bin/swiftfetch"
  
  # Install the config directory directly to ~/.config/swiftfetch (using /etc/skel as a default for system-wide installation)
  if [ -d "config" ]; then
    install -Dm644 config/config.toml "$pkgdir/etc/skel/.config/swiftfetch/config.toml"
  fi
}

# Post install hook
post_install() {
  echo "Swiftfetch package installed. Please move the default config to your home directory if needed:"
  echo "  cp /etc/skel/.config/swiftfetch/config.toml ~/.config/swiftfetch/"
}
