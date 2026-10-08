use crate::prelude::*;

use openssl::{
    hash::MessageDigest,
    pkey::PKey,
    sign::Signer,
    ssl::{Ssl, SslContextBuilder, SslMethod, SslVersion},
};
use std::{pin::Pin, time::Duration};
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

// EG4 Monitor 1.6.4's DonglePskUtil and PskTlsConfig; verified with ENC V3.03.
const PSK_IDENTITY: &[u8] = b"Client_identity\0";
const PSK_SALT: &[u8] = b"LuxPowerTek!";
const PSK_CIPHER: &str = "DHE-PSK-AES128-GCM-SHA256";

fn derive_psk(datalog: Serial) -> Result<[u8; 16]> {
    let salt = PKey::hmac(PSK_SALT)?;
    let mut signer = Signer::new(MessageDigest::sha256(), &salt)?;
    signer.update(&datalog.data())?;
    let digest = signer.sign_to_vec()?;
    Ok(digest[..16].try_into()?)
}

/// Authenticate an ENC dongle before any LuxPower packets are sent.
/// The dongle uses PSK authentication, without a certificate or a cloud login.
pub async fn connect(stream: TcpStream, datalog: Serial) -> Result<SslStream<TcpStream>> {
    let psk = derive_psk(datalog)?;
    let mut context = SslContextBuilder::new(SslMethod::tls_client())?;
    context.set_min_proto_version(Some(SslVersion::TLS1_2))?;
    context.set_max_proto_version(Some(SslVersion::TLS1_2))?;
    context.set_cipher_list(PSK_CIPHER)?;
    context.set_psk_client_callback(move |_, _, identity, key| {
        if identity.len() < PSK_IDENTITY.len() || key.len() < psk.len() {
            return Ok(0);
        }
        identity[..PSK_IDENTITY.len()].copy_from_slice(PSK_IDENTITY);
        key[..psk.len()].copy_from_slice(&psk);
        Ok(psk.len())
    });
    let mut stream = SslStream::new(Ssl::new(&context.build())?, stream)?;
    tokio::time::timeout(Duration::from_secs(5), Pin::new(&mut stream).connect())
        .await
        .map_err(|_| anyhow!("TLS-PSK handshake timed out after 5 seconds"))?
        .map_err(|error| anyhow!("TLS-PSK handshake failed: {}", error))?;
    info!(
        "inverter {}: TLS-PSK authenticated ({}, {})",
        datalog,
        stream.ssl().version_str(),
        stream
            .ssl()
            .current_cipher()
            .map_or("unknown", |c| c.name())
    );
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psk_matches_independent_hmac_vector() {
        // Python hmac.new(b"LuxPowerTek!", b"2222222222", hashlib.sha256)[:16].
        assert_eq!(
            derive_psk("2222222222".parse().unwrap()).unwrap(),
            [
                0xd2, 0x86, 0x76, 0x03, 0x44, 0x7e, 0xaa, 0x43, 0x02, 0x4b, 0x51, 0xec, 0x62, 0x2e,
                0x1c, 0xe2
            ]
        );
    }
}
