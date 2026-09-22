#!/bin/sh

set -eu

if ! command -v mkcert >/dev/null 2>&1; then
  echo "mkcert is required. Install it first: https://github.com/FiloSottile/mkcert" >&2
  exit 1
fi

project_directory=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
certificate_directory=${GITSTARS_CERT_DIR:-"$project_directory/.certs"}

mkdir -p "$certificate_directory"
mkcert -install
mkcert \
  -cert-file "$certificate_directory/localhost.pem" \
  -key-file "$certificate_directory/localhost-key.pem" \
  localhost 127.0.0.1 ::1

# 容器以非特权用户运行，需要可读的开发证书。
chmod 0644 \
  "$certificate_directory/localhost.pem" \
  "$certificate_directory/localhost-key.pem"

echo "Local HTTPS certificate created in $certificate_directory"