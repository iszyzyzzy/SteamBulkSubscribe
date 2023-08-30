use std::io;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct SteamData {
    web_apikey: String,
    file_ids: Vec<i64>,
}

#[tokio::main]
async fn main() {
    if !check_data_file_existence() {
        pause();
        return;
    }
    let data = std::fs::read_to_string("data.json").expect("Reading file error! Aborting");
    let data: SteamData = serde_json::from_str(data.as_str()).expect("Parsing json error! Aborting");
    for file_id in data.file_ids {
        match subscribe_file(file_id, &data.web_apikey).await {
            Ok(_) => println!("Successfully subscribed file {}. Proceeding", file_id),
            Err(e) => {
                println!("Error subscribing file {}, status code {}", file_id, e.to_string());
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
        std::fs::write("data.json", r#"{
  "web_apikey": "PLACE KEY HERE",
  "file_ids": [
    2669101904,
    2341745038
  ]
}"#).unwrap();
        println!(r#"Data input file not found, please place
1. All the file id you'd like to subscribe
2. Your Steam Web API key
in data.json.
An example data.json file has been generated"#);
        false
    };
}

async fn subscribe_file(file_id: i64, api_key: &String) -> Result<(), Box<dyn std::error::Error>> {
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
