use crate::config::CryptoConfig;

#[derive(Debug, Clone)]
pub enum CryptoMode {
    Plain,
    Tls { priv_key: String, pub_key: String },
}

impl From<CryptoConfig> for CryptoMode {
    fn from(config: CryptoConfig) -> Self {
        match config.crypto_type.as_str() {
            "plain" => CryptoMode::Plain,
            "tls" => CryptoMode::Tls {
                priv_key: config.priv_key.unwrap(),
                pub_key: config.pub_key.unwrap(),
            },
            _ => panic!("Unknown crypto type: {}", config.crypto_type),
        }
    }
}

use openssl::ssl::{
    SslAcceptor, SslFiletype, SslMethod, SslMode, SslOptions, SslVerifyMode, SslVersion,
};

pub fn create_ssl_acceptor(priv_key: String, pub_key: String) -> SslAcceptor {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();
    builder.set_security_level(0);
    builder
        .set_min_proto_version(Some(SslVersion::SSL3))
        .unwrap();
    builder
        .set_max_proto_version(Some(SslVersion::SSL3))
        .unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    builder.set_cipher_list("RC4-SHA").unwrap();

    // Clear all options first
    builder.clear_options(SslOptions::ALL);
    builder.clear_options(SslOptions::NO_SSLV3);
    builder.clear_options(SslOptions::ENABLE_MIDDLEBOX_COMPAT);
    builder.clear_options(SslOptions::CIPHER_SERVER_PREFERENCE);
    builder.clear_options(SslOptions::NO_TLSV1_3);
    builder.clear_options(SslOptions::NO_COMPRESSION);

    // Set reasonable options...
    builder.set_options(SslOptions::ALLOW_UNSAFE_LEGACY_RENEGOTIATION);
    builder.set_options(SslOptions::NO_COMPRESSION);
    builder.set_options(SslOptions::NO_TICKET);

    builder.set_mode(SslMode::AUTO_RETRY | SslMode::SEND_FALLBACK_SCSV);

    // Load the certificate and private key
    builder
        .set_private_key_file(priv_key, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(pub_key).unwrap();
    builder.build()
}
