mod common;
use common::*;

#[tokio::test]
async fn sends_http_request() {
    common_setup();

    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/write")
        .match_query(Matcher::UrlEncoded("db".to_owned(), "lxp".to_owned()))
        .with_status(204)
        .match_body("inputs,datalog=BA12345678 p_pv=250i,soc=100i,v_bat=52.4 1000000000")
        .create();

    let config = Factory::example_config_wrapped();
    //config.set_influx_url(server.url());
    config.influx_mut().url = server.url();
    let channels = Channels::new();

    let influx = Influx::new(config, channels.clone());

    let tf = async {
        let json =
            json!({ "time": 1, "datalog": "BA12345678", "soc": 100, "p_pv": 250, "v_bat": 52.4 });
        channels
            .to_influx
            .send(influx::ChannelData::InputData(json))?;
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !mock.matched() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await?;
        influx.stop();
        Ok(())
    };

    futures::try_join!(influx.start(), tf).unwrap();

    mock.assert();
}

#[tokio::test]
async fn skips_invalid_samples_and_recovers_from_queue_overflow() {
    let mut server = mockito::Server::new();
    common_setup();
    let mock = server
        .mock("POST", "/write")
        .match_query(Matcher::UrlEncoded("db".to_owned(), "lxp".to_owned()))
        .with_status(204)
        .create();
    let config = Factory::example_config_wrapped();
    config.influx_mut().url = server.url();
    let channels = Channels::new();
    let influx = Influx::new(config, channels.clone());
    let assertions = async {
        for _ in 0..3000 {
            channels
                .to_influx
                .send(influx::ChannelData::InputData(json!({"time": i64::MAX})))
                .unwrap();
        }
        channels
            .to_influx
            .send(influx::ChannelData::InputData(json!({
                "time": 1, "datalog": "BA12345678", "soc": 100
            })))
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while !mock.matched() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        influx.stop();
    };
    let (result, ()) = tokio::join!(influx.start(), assertions);
    result.unwrap();
    mock.assert();
}
