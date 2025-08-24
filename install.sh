#!/bin/sh
set -e
echo "Начинаем устанавливать fpm: fpm (cli) и fpmd (systemd service included)"

TMPDIR="$(mktemp -d)"
REPO=https://github.com/feproldo/process-manager
PREFIX=/usr/local
BINDIR=$PREFIX/bin
SYSTEMD_DIR=/etc/systemd/system

git clone --depth 1 "$REPO" "$TMPDIR"
cd "$TMPDIR"

cd cli
cargo build --release

cd ../daemon
cargo build --release

install -Dm755 "$TMPDIR/cli/target/release/fpm" "$BINDIR/fpm"
install -Dm755 "$TMPDIR/daemon/target/release/fpmd" "$BINDIR/fpmd"

install -Dm644 packaging/fpmd.service "$SYSTEMD_DIR/fpmd.service"

rm -rf "$TMPDIR"

systemctl daemon-reload
systemctl enable --now fpmd.service

echo "Установка завершена."
