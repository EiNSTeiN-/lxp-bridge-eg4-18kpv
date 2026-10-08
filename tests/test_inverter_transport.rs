mod common;
use common::*;

use openssl::{
    dh::Dh,
    ssl::{Ssl, SslContext, SslContextBuilder, SslMethod, SslVersion},
};
use std::{pin::Pin, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
};
use tokio_openssl::SslStream;

// Independent HMAC-SHA256 vector for the synthetic dongle 2222222222.
const TEST_PSK: [u8; 16] = [
    0xd2, 0x86, 0x76, 0x03, 0x44, 0x7e, 0xaa, 0x43, 0x02, 0x4b, 0x51, 0xec, 0x62, 0x2e, 0x1c, 0xe2,
];

fn tls_server() -> SslContext {
    let mut context = SslContextBuilder::new(SslMethod::tls_server()).unwrap();
    context
        .set_min_proto_version(Some(SslVersion::TLS1_2))
        .unwrap();
    context
        .set_max_proto_version(Some(SslVersion::TLS1_2))
        .unwrap();
    context
        .set_cipher_list("DHE-PSK-AES128-GCM-SHA256")
        .unwrap();
    context.set_tmp_dh(&Dh::get_2048_256().unwrap()).unwrap();
    context.set_psk_server_callback(|_, identity, key| {
        if identity != Some(b"Client_identity".as_slice()) || key.len() < TEST_PSK.len() {
            return Ok(0);
        }
        key[..TEST_PSK.len()].copy_from_slice(&TEST_PSK);
        Ok(TEST_PSK.len())
    });
    context.build()
}

fn holding_response() -> Vec<u8> {
    // Protocol 5 returns 80 registers (160 value bytes), as on ENC V3.03.
    let mut data = vec![0, 3];
    data.extend_from_slice(b"5555555555");
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.push(160);
    for value in 0_u16..80 {
        data.extend_from_slice(&value.to_le_bytes());
    }
    let crc = crc16::State::<crc16::MODBUS>::calculate(&data);
    let mut frame = vec![161, 26, 5, 0];
    frame.extend_from_slice(&(16_u16 + data.len() as u16).to_le_bytes());
    frame.extend_from_slice(&[1, 194]);
    frame.extend_from_slice(b"2222222222");
    frame.extend_from_slice(&(2_u16 + data.len() as u16).to_le_bytes());
    frame.extend_from_slice(&data);
    frame.extend_from_slice(&crc.to_le_bytes());
    frame
}

#[test]
fn frame_length_uses_both_bytes_and_rejects_truncated_frames() {
    let frame = holding_response();
    assert!(lxp::packet::Parser::parse(&frame).is_ok());
    assert!(lxp::packet::Parser::parse(&frame[..frame.len() - 1]).is_err());
    let mut wrong_length = frame;
    wrong_length[5] = 1;
    assert!(lxp::packet::Parser::parse(&wrong_length).is_err());
}

async fn serve<S: AsyncRead + AsyncWrite + Unpin>(mut stream: S) {
    let mut request = [0; 38];
    stream.read_exact(&mut request).await.unwrap();
    assert_eq!(&request[..8], &[161, 26, 1, 0, 32, 0, 1, 194]);
    assert_eq!(&request[8..18], b"2222222222");
    assert_eq!(request[21], 3);
    assert_eq!(&request[22..32], b"5555555555");
    assert_eq!(&request[32..36], &[0, 0, 40, 0]);
    let response = holding_response();
    stream.write_all(&response[..11]).await.unwrap();
    tokio::task::yield_now().await;
    stream.write_all(&response[11..]).await.unwrap();
    // Keep the connection open until the bridge stops; TLS may close without
    // close_notify when a task is dropped, so either EOF or an error is fine.
    let _ = stream.read(&mut [0]).await;
}

async fn roundtrip(tls: bool) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut entry = Factory::inverter();
    entry.host = "127.0.0.1".to_string();
    entry.port = listener.local_addr().unwrap().port();
    entry.tls = Some(tls);
    let config = Factory::example_config_wrapped();
    config.set_inverters(vec![entry.clone()]);
    let channels = Channels::new();
    let mut incoming = channels.from_inverter.subscribe();
    let inverter = lxp::inverter::Inverter::new(config, &entry, channels.clone());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        if tls {
            let mut stream = SslStream::new(Ssl::new(&tls_server()).unwrap(), stream).unwrap();
            Pin::new(&mut stream).accept().await.unwrap();
            assert_eq!(stream.ssl().version_str(), "TLSv1.2");
            assert_eq!(
                stream.ssl().current_cipher().unwrap().name(),
                "DHE-PSK-AES128-GCM-SHA256"
            );
            serve(stream).await;
        } else {
            serve(stream).await;
        }
    });
    let assertions = async {
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(3), incoming.recv())
                .await
                .unwrap()
                .unwrap(),
            lxp::inverter::ChannelData::Connected(entry.datalog())
        );
        // Send immediately after Connected, exercising subscription ordering.
        channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(Packet::TranslatedData(
                lxp::packet::TranslatedData {
                    datalog: entry.datalog(),
                    inverter: entry.serial(),
                    device_function: lxp::packet::DeviceFunction::ReadHold,
                    register: 0,
                    values: 40_u16.to_le_bytes().to_vec(),
                },
            )))
            .unwrap();
        let reply = tokio::time::timeout(Duration::from_secs(3), incoming.recv())
            .await
            .unwrap()
            .unwrap();
        match reply {
            lxp::inverter::ChannelData::Packet(Packet::TranslatedData(td)) => {
                assert_eq!(td.datalog, entry.datalog());
                assert_eq!(td.inverter, entry.serial());
                assert_eq!(td.values.len(), 160);
                assert_eq!(td.pairs().last(), Some(&(79, 79)));
            }
            other => panic!("Unexpected reply: {:?}", other),
        }
        inverter.stop();
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap();
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(inverter.start(), assertions)
    })
    .await
    .unwrap();
    result.unwrap();
}

#[tokio::test]
async fn plain_tcp_roundtrip_and_shutdown() {
    roundtrip(false).await;
}

#[tokio::test]
async fn tls_psk_roundtrip_and_shutdown() {
    roundtrip(true).await;
}

#[tokio::test]
async fn incorrect_dongle_serial_rejects_tls_without_connected_event() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut entry = Factory::inverter();
    entry.host = "127.0.0.1".to_string();
    entry.port = listener.local_addr().unwrap().port();
    entry.datalog = "9999999999".parse().unwrap();
    entry.tls = Some(true);
    let config = Factory::example_config_wrapped();
    config.set_inverters(vec![entry.clone()]);
    let channels = Channels::new();
    let mut incoming = channels.from_inverter.subscribe();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = SslStream::new(Ssl::new(&tls_server()).unwrap(), stream).unwrap();
        assert!(Pin::new(&mut stream).accept().await.is_err());
    });
    let inverter = lxp::inverter::Inverter::new(config, &entry, channels);
    let assertions = async {
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(3), incoming.recv())
                .await
                .unwrap()
                .unwrap(),
            lxp::inverter::ChannelData::Disconnect("9999999999".parse().unwrap())
        );
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap();
    };
    tokio::select! {
        result = inverter.start() => panic!("unexpected termination: {:?}", result),
        _ = assertions => {}
    }
}
