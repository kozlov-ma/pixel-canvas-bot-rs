#![feature(async_closure)]
use std::thread;
use std::time::Duration;
use clap::{arg, Parser};
use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::task;


mod pixel_data;
mod pixel_color;
mod requests;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
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

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();
    if args.threads <= 0 {
        panic!("Number of threads must be positive");
    }

    let requests = requests::get_requests(&args);

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
        threads.push(task::spawn_blocking(move || { requests::send_requests(c, timeout.clone(), &mut p_pix) }));
    }

    for t in threads.into_iter() {
        t.await.expect("thread panicked");
    }

    println!("Done!");
}
