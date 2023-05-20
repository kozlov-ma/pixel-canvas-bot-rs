#![feature(async_closure)]
use std::{thread};
use std::time::Duration;
use clap::Parser;
use image::Pixel;
pub use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{ParallelIterator};
use reqwest::header::HeaderMap;
use pixel_data::PixelData;
use tokio::task;


mod pixel_data;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, allow_hyphen_values=true)]
    x_pos: i32,
    #[arg(short, long, allow_hyphen_values=true)]
    y_pos: i32,

    #[arg(short, long)]
    threads: i32,

    #[arg(short, long)]
    image_path: String,

    #[arg(short, long)]
    fingerprint: String,

    #[arg(long)]
    invert: bool,

    #[arg(long)]
    grayscale: bool,

    #[arg(long)]
    shuffle: bool,

    #[arg(long)]
    reverse: bool,

    #[arg(long, default_value_t = 65536)]
    timeout: u64,
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent",
                   "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0)\
                    Gecko/20100101 Firefox/113.0"
                       .parse().unwrap());

    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Accept-Language", "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3.png".parse().unwrap());
    headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    headers.insert("Referer", "https://pixel.alexbers.com/@41,129".parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Origin", "https://pixel.alexbers.com".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "cors".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());

    headers
}

fn get_requests(x_pos: i32, y_pos: i32, grayscale: bool, invert: bool, shuffle: bool,
                reverse: bool, fingerprint: &str, image_path: &str) -> Vec<String> {
    use rand::{thread_rng, seq::SliceRandom};
    let mut img = image::open(image_path).expect("No image found by the given path.");
    match invert {
        true => img.invert(),
        false => ()
    };

    let pixels = match grayscale {
        true => img.grayscale().to_rgba8(),
        false => img.to_rgba8()
    };

    let pixels = pixels
        .enumerate_pixels()
        .filter(|(_x, _y, p)| p.channels()[3] > 0)
        .map(|(x, y, p)| (x, y, p.to_rgb()));

    let pixels = pixels
        .map(
            |(x, y, p)|
                PixelData::from_rgb8(x as i32 + x_pos, y as i32 + y_pos, &p, grayscale));

    let mut requests: Vec<String> = pixels
        .map(|p| p.get_json(fingerprint))
        .collect();

    if shuffle {
        requests.shuffle(&mut thread_rng());
    }

    if reverse {
        requests.reverse();
    }

    requests
}


async fn send_request(url: &str, request: String, headers: Arc<HeaderMap>, client: Arc<reqwest::Client>, timeout_ms: u64, progress: &mut Arc<Mutex<ProgressBar>>) {
    let headers= headers.as_ref();
    if let Err(r) = client.post(url)
        .body(request.clone())
        .headers(headers.to_owned())
        .timeout(Duration::from_millis(timeout_ms))
        .send()
        .await { println!("Error, request: {}", r) };

    let mut pixels = progress.lock().unwrap();
    pixels.inc(1);
}

#[tokio::main]
async fn send_requests(url: &str, requests: Vec<String>, timeout_ms: u64, progress: &mut Arc<Mutex<ProgressBar>>){
    let client = Arc::new(reqwest::Client::new());
    let headers = Arc::new(get_headers());

    for r in requests.into_iter() {
        let fut = send_request(
            url,
            r.to_owned(),
            headers.clone(),
            client.clone(),
            timeout_ms.clone(),
            progress
        );

        fut.await;
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();
    if args.threads <= 0 {
        panic!("Number of threads must be positive");
    }

    let url = "https://pixel.alexbers.com/api/pixel";
    let requests
        = get_requests(
            args.x_pos,
            args.y_pos,
            args.grayscale,
            args.invert,
            args.shuffle,
            args.reverse,
            &args.fingerprint,
            &args.image_path);

    println!("Starting to print in three seconds...");
    thread::sleep(Duration::from_secs(3));
    println!("Sending reqwests...");

    let req_len = requests.len();
    let chunk_length = &req_len / args.threads as usize;
    let chunks: Vec<Vec<String>> = requests.chunks(chunk_length).map(|s| s.into()).collect();

    let style
        = ProgressStyle::with_template("[{elapsed_precise}]\
         [{per_sec}]\
          [{eta_precise}]\
          {bar:40.cyan/blue}\
           {pos:>7}/{len:7}\
            {msg}").expect("Wrong progress style");

    let progress = Arc::new(Mutex::new(ProgressBar::new(req_len as u64).with_style(style)));

    let mut threads = vec![];

    let timeout = args.timeout;
    for c in chunks {
        let mut p_pix = progress.clone();
        threads.push(task::spawn_blocking(move || { send_requests(url, c, timeout.clone(), &mut p_pix) }));
    }

    for t in threads.into_iter() {
        t.await.expect("thread panicked");
    }

    println!("Done!");
}
