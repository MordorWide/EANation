#!/usr/bin/env bash

# OpenSSL version:
# $> openssl version
# OpenSSL 3.0.2 15 Mar 2022 (Library: OpenSSL 3.0.2 15 Mar 2022)

# Remove old certificates
rm priv.pem
rm pub.pem

# Generate Cert
openssl genrsa -out priv.pem 1024
# Generate CSR
openssl req -new -key priv.pem -out request.csr -subj "/C=US/ST=California/L=Redwood City/O=Electronic Arts, Inc./OU=Online Technology Group/CN=fesl.ea.com/emailAddress=admin@mordorwi.de"

# Make a CA
openssl genrsa -out ca.key 1024
openssl req -new -x509 -days 3650 -key ca.key -out ca.crt -subj "/CN=OTG3 Certificate Authority/C=US/ST=California/L=Redwood City/O=Electronic Arts, Inc./OU=Online Technology Group/emailAddress=admin@mordorwi.de"

cat > cert.cnf <<EOF
[ v3_ca ]
basicConstraints = critical,CA:FALSE
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always
EOF

# Sign the CSR
OPENSSL_ENABLE_SHA1_SIGNATURES=1 openssl x509 -req \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -sha1 \
    -days 3650 \
    -in request.csr \
    -extfile cert.cnf \
    -extensions v3_ca \
    -out pub.pem

# Cleanup
rm request.csr
rm cert.cnf
rm ca.srl
rm ca.key
rm ca.crt
