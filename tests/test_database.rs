mod common;
use common::*;

use {futures::TryStreamExt, sqlx::Row};

// avoids having to specify return types for row.get()
fn assert_str_eq(input: &str, expected: &str) {
    assert_eq!(input, expected);
}
fn assert_u16_eq(input: i32, expected: u16) {
    assert_eq!(input as u16, expected);
}
fn assert_i16_eq(input: i32, expected: i16) {
    assert_eq!(input as i16, expected);
}
fn assert_i32_eq(input: i32, expected: i32) {
    assert_eq!(input, expected);
}
fn assert_f64_eq(input: f64, expected: f64) {
    assert_eq!(input, expected);
}

#[tokio::test]
async fn sqlite_insertion() {
    common_setup();

    let config = config::Database {
        enabled: true,
        url: "sqlite::memory:".to_string(),
    };
    let channels = Channels::new();

    let database = Database::new(config, channels.clone());

    let tf = async {
        let channel_data = database::ChannelData::ReadInputAll(Box::new(Factory::read_input_all()));

        // Consumers subscribe before initialization; queue acceptance does not mean
        // the pool and migrations are ready. Wait for the actual table instead.
        let mut conn = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                if let Ok(mut conn) = database.connection().await {
                    if sqlx::query("SELECT * FROM inputs")
                        .fetch_optional(&mut conn)
                        .await
                        .is_ok()
                    {
                        break conn;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await?;
        channels.to_database.send(channel_data)?;

        let mut retries = 0;
        loop {
            let mut rows = sqlx::query("SELECT * FROM inputs").fetch(&mut conn);
            if let Some(row) = rows.try_next().await? {
                let ria = Factory::read_input_all();
                // really this should test a whole lot more columns, but tedious
                assert_u16_eq(row.get("status"), ria.status);
                assert_i32_eq(row.get("p_grid"), ria.p_grid);
                assert_i32_eq(row.get("p_battery"), ria.p_battery);
                assert_u16_eq(row.get("p_discharge"), ria.p_discharge);
                assert_f64_eq(row.get("e_to_user_day"), ria.e_to_user_day);
                assert_f64_eq(row.get("e_pv_all"), ria.e_pv_all);
                assert_i16_eq(row.get("t_rad_2"), ria.t_rad_2);
                assert_u16_eq(row.get("bms_event_1"), ria.bms_event_1);
                assert_u16_eq(row.get("bms_event_2"), ria.bms_event_2);
                assert_u16_eq(row.get("bms_fw_update_state"), ria.bms_fw_update_state);
                assert_u16_eq(row.get("cycle_count"), ria.cycle_count);
                assert_f64_eq(row.get("vbat_inv"), ria.vbat_inv);
                assert_str_eq(row.get("datalog"), "1234567890");
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            retries = retries + 1;

            if retries > 50 {
                panic!("row not inserted");
            }
        }

        database.stop();
        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(database.start(), tf).unwrap();
}

#[tokio::test]
async fn recovers_when_database_is_unavailable_at_startup() {
    common_setup();
    let directory =
        std::env::temp_dir().join(format!("lxp-database-recovery-{}", std::process::id()));
    assert!(!directory.exists());
    let config = config::Database {
        enabled: true,
        url: format!(
            "sqlite://{}?mode=rwc",
            directory.join("inputs.db").display()
        ),
    };
    let channels = Channels::new();
    let database = Database::new(config, channels.clone());
    let assertions = async {
        // The first connection fails because the directory does not exist.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        std::fs::create_dir(&directory).unwrap();
        channels
            .to_database
            .send(database::ChannelData::ReadInputAll(Box::new(
                Factory::read_input_all(),
            )))
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(25), async {
            loop {
                if let Ok(mut conn) = database.connection().await {
                    if let Ok(Some(_)) = sqlx::query("SELECT * FROM inputs")
                        .fetch_optional(&mut conn)
                        .await
                    {
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        database.stop();
    };
    let (result, ()) = tokio::join!(database.start(), assertions);
    result.unwrap();
    drop(database);
    std::fs::remove_dir_all(directory).unwrap();
}
