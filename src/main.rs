extern crate clap;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;

use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use simple_user_input::get_input;
use tokio::prelude::*;


async fn request_hlp(count: u64, animal: &str) -> Vec<String> {
    let res = reqwest::get(&format!(
        "http://shibe.online/api/{}?count={}&urls=true&httpsUrls=true",
        animal, count
    ))
    .await
    .expect("request failed");
    let body: String = res.text().await.expect("failed to get response");
    let body_json: Vec<String> = serde_json::from_str(&body).unwrap();
    return body_json;
}

async fn first_request(count: String, animal: &str) -> Vec<String> {
    let count_i: u64 = count.parse().unwrap();
    let requests_i: u64;
    let mut body_p: Vec<String> = Vec::new();
    let mut leftover_i: u64 = 0;
    if count_i % 100 == 0 {
        requests_i = (count_i as f64 / 100_f64).ceil() as u64;
        leftover_i = 0;
    } else if count_i % 100 != 0 {
        let rq_i = count_i as f64 / 100_f64;
        requests_i = rq_i.ceil() as u64 - 1;
        leftover_i = count_i % 100;
    } else {
        requests_i = 0
    }
    if requests_i >= 1 {
        for _i in 0..requests_i {
            let body_json: Vec<String> = request_hlp(100, animal).await;
            body_p.extend(body_json);
        }
        if leftover_i > 0 {
            let body_json: Vec<String> = request_hlp(leftover_i, animal).await;
            body_p.extend(body_json);
        }
    } else {
        let body_json: Vec<String> = request_hlp(count_i, animal).await;
        body_p.extend(body_json);
    }
    return body_p;
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let matches =
        App::new("Animols downloader")
            .version("1.0")
            .author("YaAlex <yaalex@yaalex.tk>")
            .about("Downloads cute animols from shibe.online")
            .arg(Arg::with_name("count").about("Count of animols to download"))
            .arg(Arg::with_name("anim").about(
                "Which animol pics to download\ns - shibe\nb - birb\nc - catto\nDefault - shib",
            ))
            .arg(
                Arg::new("debug")
                    .short('d')
                    .about("print debug information verbosely"),
            )
            .get_matches();
    let count: String;
    if matches.is_present("debug") {
        println!("Printing debug info...");
    }
    if let Some(cnt) = matches.value_of_lossy("count") {
        count = cnt.to_string();
    } else {
        count = get_input("How many pics?\nDefault - 1");
    }

    let animal: &str;
    let anim: String;
    if let Some(anm) = matches.value_of_lossy("anim") {
        anim = anm.to_string();
    } else {
        anim = get_input("Which animol?\ns - shibe\nb - birb\nc - catto\nDefault - shib");
    }
    let str_anim: &str = anim.as_str();
    match str_anim {
        "s" | "shiba" | "shibe" | "shib" => animal = "shibes",
        "b" | "birb" | "bird" => animal = "birds",
        "c" | "cat" | "catto" | "kat" => animal = "cats",
        &_ => animal = "shibes",
    }

    std::fs::create_dir_all(animal).expect("failed to create dir");
    let mut i: u64 = 0;
    let mut threads: Vec<_> = Vec::new();
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(180);
    bar.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}"),
    );
    bar.set_message("Downloading...");
    let body_p = first_request(count, animal).await;
    for uri in body_p {
        i += 1;
        let thrd = tokio::task::spawn(async move {
            let mut respons = reqwest::get(&uri).await.expect("dload error");
            let fname = respons
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");

            let mut file = tokio::fs::File::create(format!("{}/{}", animal, fname))
                .await
                .expect("create error");
            while let Some(chunk) = respons.chunk().await.expect("chunk err") {
                file.write_all(&chunk).await.expect("write error");
            }
        });
        threads.push(thrd);
    }
    let mut tn: u64 = 0;
    for t in threads {
        tn += 1;
        if matches.is_present("debug") {
            println!("Started task {}, waiting...", tn);
        }
        t.await.unwrap();
    }
    bar.finish_with_message(
        format!(
            "Success! Downloaded {} {}\nLocated under ./{}",
            i, animal, animal
        )
        .as_str(),
    );
    Ok(())
}

mod simple_user_input {
    use std::io;
    pub fn get_input(prompt: &str) -> String {
        println!("{}", prompt);
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        input.trim().to_string()
    }
}
