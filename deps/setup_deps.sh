#!/usr/bin/env bash
set -e
cd "$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Setup the dependencies
if [ ! -d "openssl" ]; then
    echo "Setting up OpenSSL..."
    git clone -b OpenSSL_1_1_1g https://github.com/openssl/openssl openssl
fi

if [ ! -f "openssl/.configured" ]; then
    cd openssl
    ./Configure linux-x86_64 enable-weak-ssl-ciphers enable-deprecated enable-ssl3 enable-ssl3-method no-shared -fPIC

    touch .configured
    cd ..
fi

if [ ! -f "openssl/.built" ]; then
    cd openssl
    make -j$(nproc)

    touch .built
    cd ..
fi