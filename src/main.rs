use serde::{Deserialize, Serialize};
use std::io::Read;
use std::{io, vec};

#[derive(Serialize, Deserialize, Debug)]
struct SteamData {
    web_apikey: String,
    game_appid: i64,
    cookie: String,
    username: String,
}

#[tokio::main]
async fn main() {
    if !check_data_file_existence() {
        pause();
        return;
    }
    let data = std::fs::read_to_string("data.json").expect("Reading file error! Aborting");
    let data: SteamData =
        serde_json::from_str(data.as_str()).expect("Parsing json error! Aborting");
    let file_ids = get_subscribe_list(data.username, data.game_appid, data.cookie)
        .await
        .unwrap();
    for (index, file_id) in file_ids.iter().enumerate() {
        match subscribe_file(file_id, &data.web_apikey).await {
            Ok(_) => println!("Successfully subscribed file {}. Proceeding {}/{}", file_id, index + 1, file_ids.len()),
            Err(e) => {
                println!(
                    "Error subscribing file {}, status code {}",
                    file_id,
                    e.to_string()
                );
                pause()
            }
        }
    }
    println!("All provided file id subscribed successfully. Finishing");
    pause();
    return;
}

fn check_data_file_existence() -> bool {
    return if std::fs::metadata("data.json").is_ok() {
        println!("data.json file found. Proceeding");
        true
    } else {
        std::fs::write(
            "data.json",
            r#"{
  "web_apikey": "PLACE KEY HERE",
  "game_appid": 0,
  "cookie": "PLACE COOKIE HERE",
  "username": "PLACE USER NAME HERE"
}"#,
        )
        .unwrap();
        println!(
            r#"Data input file not found, please place
1.web_apikey of the receiving account
2.appid of the game to be transferred
3.cookie of the sending account (focus on steamLoginSecure)
4.username of the sending account
in data.json.
An example data.json file has been generated"#
        );
        false
    };
}

async fn subscribe_file(file_id: &i64, api_key: &String) -> Result<(), Box<dyn std::error::Error>> {
    let status_code = reqwest::Client::new()
        .post(format!("https://api.steampowered.com/IPublishedFileService/Subscribe/v1/?key={}&publishedfileid={}&list_type=1", api_key, file_id))
        .header("content-length", 0)
        .send()
        .await.unwrap().status();
    return if status_code.is_success() {
        Ok(())
    } else {
        println!("Request Steam API failed! Aborting");
        Err(status_code.as_str().into())
    };
}

fn pause() {
    let mut stdin = io::stdin();
    let _ = stdin.read(&mut [0u8]).unwrap();
}

async fn get_subscribe_list(
    username: String,
    game_appid: i64,
    cookie: String,
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let mut result: Vec<i64> = vec![];
    let mut page = 1;
    loop {
        let response = reqwest::Client::new()
            .get(format!("https://steamcommunity.com/id/{}/myworkshopfiles/?appid={}&browsefilter=mysubscriptions&p={}&numperpage=30", username, game_appid, page))
            .header("cookie", &cookie)
            .send()
            .await.unwrap().text().await.unwrap();
        let doc = scraper::Html::parse_document(response.as_str());
        let selector = scraper::Selector::parse("div[id^='Subscription']").unwrap();
        let ids: Vec<i64> = doc
            .select(&selector)
            .filter_map(|element| {
                // 获取 id 属性，去掉前缀 "Subscription"
                element
                    .value()
                    .id()
                    .map(|id| id.trim_start_matches("Subscription").to_string())
            })
            .map(|id| id.parse::<i64>().unwrap())
            .collect();
        if ids.is_empty() {
            break;
        }
        result.extend(ids);
        print!("Page {} done.  {} items collected\n", page, result.len());
        page += 1;
    }
    print!("All pages done. Total {} items collected\n", result.len());
    return Ok(result);
}
