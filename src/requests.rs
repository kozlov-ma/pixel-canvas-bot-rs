use std::string::ToString;
use std::sync::{Arc, Mutex};
use reqwest::header::HeaderMap;
use indicatif::ProgressBar;
use std::time::Duration;
pub use image::{Pixel};
use crate::Args;
use crate::pixel_data::PixelData;

pub static  URL: &str = "https://pixel.alexbers.com/api/pixel";

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

pub fn get_requests(args: &Args)-> Vec<String> {
    use rand::{thread_rng, seq::SliceRandom};

    let mut img = image::open(&args.image_path)
        .expect(&format!("No image found at {}", &args.image_path));

    if args.invert {
        img.invert();
    }

    if args.grayscale {
        img = img.grayscale();
    }

    let img = img.to_rgba8();

    let pixels = img
        .enumerate_pixels()
        .filter(|(_, _, p)| p.channels()[3] > 0)
        .map(|(x, y, p)| (x, y, p.to_rgb()))
        .map(|(x, y, p)| PixelData::from_rgb8(x as i32 + args.x_pos.clone(),
                                              y as i32 + args.y_pos.clone(),
                                              &p, args.grayscale.clone()));

    let mut requests: Vec<String> = pixels
        .map(|p| p.get_json(&args.fingerprint))
        .collect();

    if args.shuffle {
        requests.shuffle(&mut thread_rng());
    }

    if args.reverse {
        requests.reverse();
    }

    requests
}

async fn send_request(request: String, headers: Arc<HeaderMap>, client: Arc<reqwest::Client>, timeout_ms: u64, progress: &mut Arc<Mutex<ProgressBar>>) {
    let headers= headers.as_ref();
    if let Err(r) = client.post(URL)
        .body(request.clone())
        .headers(headers.to_owned())
        .timeout(Duration::from_millis(timeout_ms))
        .send()
        .await { println!("Error, request: {}", r) };

    let pixels = progress.lock().unwrap();
    pixels.inc(1);
}

#[tokio::main]
pub async fn send_requests(requests: Vec<String>, timeout_ms: u64, progress: &mut Arc<Mutex<ProgressBar>>){
    let client = Arc::new(reqwest::Client::new());
    let headers = Arc::new(get_headers());

    for r in requests.into_iter() {
        let fut = send_request(
            r.to_owned(),
            headers.clone(),
            client.clone(),
            timeout_ms.clone(),
            progress
        );

        fut.await;
    }
}
