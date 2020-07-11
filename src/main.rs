extern crate serde_json;
extern crate reqwest;
extern crate tokio;
extern crate clap;

use clap::{App, Arg};
use simple_user_input::get_input;
use tokio::prelude::*;
use std::io;
use std::io::prelude::*;

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let matches = App::new("Animols downloader")
        .version("1.0")
        .author("YaAlex <yaalex@yaalex.tk>")
        .about("Downloads cute animols from shibe.online")
        .arg(Arg::with_name("count")
            .about("Count of animols to download")
            .multiple(true))
        .arg(Arg::new("debug")
            .short('d')
            .about("print debug information verbosely"))
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
    let _shib: String = String::from("s");
    let _birb: String = String::from("b");
    let _cat: String = String::from("c");
    let anim = get_input("Which animol?\ns - shibe\nb - birb\nc - catto\nDefault - shib");
    let str_anim: &str = anim.as_str();
    match str_anim {
        "s" | "shiba" | "shibe" | "shib" => animal = "shibes",
        "b" | "birb" | "bird" => animal = "birds",
        "c" | "cat" | "catto" | "kat" => animal = "cats",
        &_ => animal = "shibes",
    }

    let url: String = format!("http://shibe.online/api/{}?count={}&urls=true&httpsUrls=true", animal, count);
    let res = reqwest::get(&url).await.expect("request failed");
    if matches.is_present("debug") {
        println!("Status: {}", res.status());
    }
    let body: String = res.text().await.expect("failed to get response");
    let body_json = serde_json::from_str(&body).unwrap();
    let body_p: Vec<String> = body_json;
    std::fs::create_dir_all(animal).expect("failed to create dir");
    let mut i: u8 = 0;
    let mut threads: Vec<_> = Vec::new();
    for uri in body_p { 
        i += 1;
        let thrd = tokio::task::spawn(async move {
            let mut respons = reqwest::get(&uri).await.expect("dload error");    
            let fname = format!("{}/{}_{}.jpg", animal, animal, i);
            let mut file = tokio::fs::File::create(fname).await.expect("create error");
            while let Some(chunk) = respons.chunk().await.expect("chunk err") {
                file.write_all(&chunk).await.expect("write error");
            }
        });
        threads.push(thrd);
    }
    let mut tn: u8 = 0;
    for t in threads {
        tn += 1;
        if matches.is_present("debug") {
            println!("Started task {}, waiting...", tn);
        }
        t.await.unwrap();
    }
    println!("Success! Downloaded {} {}\nLocated under ./{}", i, animal, animal);
    pause();
    Ok(())
}

mod simple_user_input {
    use std::io;
    pub fn get_input(prompt: &str) -> String{
        println!("{}",prompt);
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {},
            Err(_no_updates_is_fine) => {},
        }
        input.trim().to_string()
    }
}