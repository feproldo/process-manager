#!/bin/sh
set -e

PREFIX=/usr/local
BINDIR=$PREFIX/bin
SYSTEMD_DIR=/etc/systemd/system
VERSION=0.0.2

BASE_URL="https://github.com/feproldo/process-manager/releases/download/v$VERSION"

curl -L "$BASE_URL/fpm" -o "$BINDIR/fpm"
curl -L "$BASE_URL/fpmd" -o "$BINDIR/fpmd"
curl -L "$BASE_URL/fpmd.service" -o "$SYSTEMD_DIR/fpmd.service"

chmod +x "$BINDIR/fpm" "$BINDIR/fpmd"

systemctl daemon-reload
systemctl enable --now fpmd.service

echo "Установка завершена."
