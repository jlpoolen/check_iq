// 2025-06-05 ChatGPT - $Header$
//
// Enhanced RTL-SDR .iq file scanner with per-second clipping breakdown, fast --break_out mode,
// and optional UTC/localtime epoch labeling.

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::process;
use chrono::{TimeZone, Utc, Local};

const BUFFER_SIZE: usize = 1024 * 1024 * 64; // 64 MB

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage_and_exit(&args[0]);
    }

    let filename = &args[1];
    let sample_rate: f64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error: sample_rate must be a floating-point number.");
        process::exit(1);
    });

    let mut break_out_second: Option<u64> = None;
    let mut epoch_utc: Option<i64> = None;
    let mut output_localtime = false;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--break_out" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --break_out requires an argument.");
                    process::exit(1);
                }
                break_out_second = args[i].parse().ok();
            }
            "--epoch_UTC" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --epoch_UTC requires an argument.");
                    process::exit(1);
                }
                epoch_utc = args[i].parse().ok();
            }
            "--output_localtime" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --output_localtime requires 'true' or 'false'.");
                    process::exit(1);
                }
                output_localtime = args[i] == "true";
            }
            unknown => {
                eprintln!("Error: unrecognized option '{}'", unknown);
                print_usage_and_exit(&args[0]);
            }
        }
        i += 1;
    }

    let file = File::open(filename).unwrap_or_else(|e| {
        eprintln!("Error opening file '{}': {}", filename, e);
        process::exit(1);
    });

    if let Some(sec) = break_out_second {
        run_breakout(file, sample_rate, sec);
    } else {
        run_full_analysis(file, sample_rate, epoch_utc, output_localtime);
    }
}

fn run_breakout(mut file: File, sample_rate: f64, second: u64) {
    let samples_per_sec = sample_rate as u64;
    let start_byte = second * samples_per_sec * 2;
    let bytes_to_read = samples_per_sec * 2;

    let file_len = file.metadata().unwrap().len();
    if start_byte + bytes_to_read > file_len {
        let max_secs = file_len as f64 / (sample_rate * 2.0);
        eprintln!(
            "Error: second {} is beyond file length (max duration: {:.2} seconds)",
            second, max_secs
        );
        process::exit(1);
    }

    file.seek(SeekFrom::Start(start_byte)).unwrap();
    let mut buffer = vec![0u8; bytes_to_read as usize];
    file.read_exact(&mut buffer).unwrap();

    let mut i_0 = 0;
    let mut i_255 = 0;
    let mut q_0 = 0;
    let mut q_255 = 0;

    for pair in buffer.chunks_exact(2) {
        if pair[0] == 0 { i_0 += 1; }
        if pair[0] == 255 { i_255 += 1; }
        if pair[1] == 0 { q_0 += 1; }
        if pair[1] == 255 { q_255 += 1; }
    }

    let total = i_0 + i_255 + q_0 + q_255;

    println!("--- Detailed Clipping for Second {} ---", second);
    println!("second {:>6}: {} clipped samples", second, total);
    println!("         I: 0:{:<6} 255:{:<6}", i_0, i_255);
    println!("         Q: 0:{:<6} 255:{:<6}", q_0, q_255);
}

fn run_full_analysis(file: File, sample_rate: f64, epoch_utc: Option<i64>, output_localtime: bool) {
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let mut i_low = 0u64;
    let mut i_high = 0u64;
    let mut q_low = 0u64;
    let mut q_high = 0u64;
    let mut total_pairs = 0u64;

    let mut clipping_per_second: HashMap<u64, u64> = HashMap::new();

    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        for (index, pair) in buffer[..bytes_read].chunks_exact(2).enumerate() {
            let i = pair[0];
            let q = pair[1];
            let sample_index = total_pairs + index as u64;
            let time_sec = (sample_index as f64 / sample_rate).floor() as u64;

            let mut clipped = false;

            if i == 0 { i_low += 1; clipped = true; }
            else if i == 255 { i_high += 1; clipped = true; }

            if q == 0 { q_low += 1; clipped = true; }
            else if q == 255 { q_high += 1; clipped = true; }

            if clipped {
                *clipping_per_second.entry(time_sec).or_insert(0) += 1;
            }
        }

        total_pairs += (bytes_read / 2) as u64;
    }

    println!("File: {}", env::args().nth(1).unwrap());

    if let Some(epoch) = epoch_utc {
        if output_localtime {
            let dt = Local.timestamp_opt(epoch, 0).unwrap();
            println!("{}", dt.format("%A, %B %e, %Y at %-I:%M %p %Z"));
        } else {
            let dt = Utc.timestamp_opt(epoch, 0).unwrap();
            println!("{}", dt.format("%A, %B %e, %Y at %H:%M UTC"));
        }
    }

    println!("Total I/Q pairs processed: {}", total_pairs);
    println!("--- Clipping Statistics ---");
    println!("I = 0     : {:>10}", i_low);
    println!("I = 255   : {:>10}", i_high);
    println!("Q = 0     : {:>10}", q_low);
    println!("Q = 255   : {:>10}", q_high);

    let total_clipped = i_low + i_high + q_low + q_high;
    let total_samples = total_pairs * 2;
    let percent = 100.0 * (total_clipped as f64) / (total_samples as f64);
    println!("Clipping percentage: {:.6}%", percent);

    println!("\n--- Clipping per second ---");
    let mut sorted: Vec<_> = clipping_per_second.into_iter().collect();
    sorted.sort_by_key(|&(sec, _)| sec);

    for (sec, count) in sorted {
        if let Some(epoch) = epoch_utc {
            if output_localtime {
                let dt = Local.timestamp_opt(epoch + sec as i64, 0).unwrap();
                println!("{}: {} clipped samples", dt.format("%H:%M:%S"), count);
            } else {
                println!("{}: {} clipped samples", epoch + sec as i64, count);
            }
        } else {
            println!("second {:>6}: {} clipped samples", sec, count);
        }
    }
}

fn print_usage_and_exit(program: &str) {
    eprintln!("Usage: {} <file.iq> <sample_rate> [--break_out N] [--epoch_UTC N] [--output_localtime true]", program);
    process::exit(1);
}
