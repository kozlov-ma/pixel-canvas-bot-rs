#![feature(async_closure)]
use std::{thread};
use std::future::IntoFuture;
use std::rc::Rc;
use futures;
use std::time::Duration;
use clap::Parser;
use image::{DynamicImage, Rgb};
use image::GenericImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::Pixel;
use image::RgbImage;
pub use rayon::prelude::*;
use strum::{IntoEnumIterator};
use std::sync::Arc;
use futures::TryFutureExt;
use indicatif::{MultiProgress, ProgressBar, ProgressIterator};
use indicatif::ParallelProgressIterator;
use indicatif::ProgressStyle;
use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use reqwest::ClientBuilder;
use reqwest::header::HeaderMap;
use pixel_data::PixelData;
use tokio::task;
use tokio::task::{JoinSet};

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

fn get_requests(x_pos: i32, y_pos: i32, grayscale: bool, inverse: bool, fingerprint: &str, image_path: &str) -> Vec<String> {
    use rand::{thread_rng, seq::SliceRandom};
    let mut img = image::open(image_path).expect("No image found by the given path.");
    match inverse {
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

    requests.shuffle(&mut thread_rng());
    requests
}


async fn send_request(url: &str, request: String, headers: Arc<HeaderMap>, client: Arc<reqwest::Client>) {
    let headers= headers.as_ref();
    if let Err(r) = client.post(url)
        .body(request.clone())
        .headers(headers.to_owned())
        // .timeout(Duration::from_millis(2000))
        .send()
        .await { println!("Error, request: {}", r) };
}

#[tokio::main]
async fn send_requests(url: &str, requests: Vec<String>){
    let client = Arc::new(reqwest::Client::new());
    let headers = Arc::new(get_headers());

    for r in requests.into_iter() {
        let fut = send_request(
            url,
            r.to_owned(),
            headers.clone(),
            client.clone()
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
        = get_requests(args.x_pos, args.y_pos, args.grayscale, args.invert, &args.fingerprint, &args.image_path);

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
    // let progress = Arc::new(ProgressBar::new(req_len as u64).with_style(style));

    let mut threads = vec![];

    for c in chunks {
        threads.push(tokio::task::spawn_blocking(|| { send_requests(url, c) }));
    }

    for t in threads.into_iter() {
        t.await.expect("thread panicked");
    }
    //
    // let headers = Arc::new(get_headers());
    // let client = Arc::new(reqwest::Client::new());
    // for r in requests.into_iter() {
    //     let r = r.to_owned();
    //     let headers = headers.clone();
    //     let client = client.clone();
    //     send_requests(url, r).await;
    //     threads.push(tokio::spawn(async move { send_request(url, r, headers, client).await }));
    // }
    //
    //
    // let style
    //     = ProgressStyle::with_template("[{elapsed_precise}]\
    //      [{per_sec}]\
    //       [{eta_precise}]\
    //       {bar:40.cyan/blue}\
    //        {pos:>7}/{len:7}\
    //         {msg}").expect("Wrong progress style");
    //
    // // task::yield_now().await;
    // for t in threads.into_iter().progress_with_style(style) {
    //     t.await.expect("Couldn't join threads");
    // }
    // send_requests(url, requests).await;

    // futures::future::join_all(threads);

    println!("Done!");
}
