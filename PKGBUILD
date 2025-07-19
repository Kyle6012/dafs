# Maintainer: Your <bahatikylemeshack@gmail.coms>
pkgname=dafs
pkgver=0.1.0
pkgrel=1
pkgdesc="Decentralized AI File System with integrated web dashboard"
arch=('x86_64')
url="https://github.com/Kyle6012/dafs"
license=('MIT')
depends=('nodejs>=18.0.0' 'npm')
makedepends=('rust' 'cargo' 'git')
source=("git+$url.git")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/$pkgname"
    # Install web dashboard dependencies
    cd web
    npm install
}

build() {
    cd "$srcdir/$pkgname"
    
    cargo build --release
}

package() {
    cd "$srcdir/$pkgname"
    
    # Install binary
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Install web assets
    install -dm755 "$pkgdir/usr/share/$pkgname/web-assets"
    cp -r target/web-assets/* "$pkgdir/usr/share/$pkgname/web-assets/"
    
    # Install systemd service
    install -Dm644 "scripts/dafs.service" "$pkgdir/usr/lib/systemd/system/dafs.service"
    
    # Install documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 QUICKSTART.md "$pkgdir/usr/share/doc/$pkgname/QUICKSTART.md"
    install -Dm644 docs/API.md "$pkgdir/usr/share/doc/$pkgname/API.md"
    
    # Install license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    
    # Create data directories
    install -dm755 "$pkgdir/var/lib/$pkgname"
    install -dm755 "$pkgdir/var/lib/$pkgname/files"
    install -dm755 "$pkgdir/var/lib/$pkgname/userkeys"
    install -dm755 "$pkgdir/var/lib/$pkgname/db"
} 