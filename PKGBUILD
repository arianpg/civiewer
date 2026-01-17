# Maintainer: arianpg <programmer.arian@gmail.com>
pkgname=civiewer
pkgver=0.1.0
pkgrel=1
pkgdesc="A fast and minimal comic image viewer."
arch=('x86_64')
url="https://github.com/arianpg/civiewer"
license=('MIT')
depends=('gtk4')
makedepends=('cargo' 'git')
source=()
sha256sums=()
options=('!strip')

build() {
    # We are building from local source, so we use startdir
    cd "$startdir"
    cargo build --release
}

package() {
    cd "$startdir"
    install -Dm755 target/release/civiewer "$pkgdir/usr/bin/$pkgname"
    install -Dm644 assets/civiewer.desktop "$pkgdir/usr/share/applications/$pkgname.desktop"
    install -Dm644 assets/hicolor/scalable/apps/com.arianpg.civiewer.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/com.arianpg.civiewer.svg"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 ThirdPartyNotices.txt "$pkgdir/usr/share/licenses/$pkgname/ThirdPartyNotices.txt"
}
