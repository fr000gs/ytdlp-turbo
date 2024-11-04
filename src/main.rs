use std::io::{self, Write};
use std::process::Command;
//use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    loop {
        // Ask if the user wants to use the same format for playlist
        run_command(vec!["color", "f0"]);
        println!("Same format for playlist? (Y/n): ");
        io::stdout().flush().unwrap();
        let use_playlist_format = get_input_with_timeout(3).unwrap_or("y".to_string()).to_lowercase();
        let is_playlist = use_playlist_format == "y";

        // Get the URL from the user
        let url = read_input("Enter URL: ");

        if is_playlist {
            download_playlist(&url);
        } else {
            download_video(&url);
        }
    }
}

/*
fn download_playlist(url: &str) {
    println!("Fetching playlist format options...");
    run_command(vec!["yt-dlp", "--color", "never", "-I", "1", "-F", url]);

    let format_code = select_format();
    println!("Selected format: {}. Starting playlist download...", format_code);

    // Get the total video count in the playlist
    let total_videos = get_playlist_count(url);
    println!("Total videos in playlist: {}", total_videos);

    // Get download range from user
    let (start_index, end_index) = get_download_range(total_videos);

    // Create a channel for thread communication
    let (tx, _rx) = mpsc::channel();

    // Ask the user for the number of concurrent downloads
    let num_threads: usize = read_input("Enter number of concurrent downloads: ")
        .parse()
        .unwrap_or(1); // Default to 1 if parsing fails

    let segment_length = (end_index - start_index + 1) / num_threads;
    let mut handles = vec![];

    for i in 0..num_threads {
        let tx = tx.clone();
        let segment_start = start_index + i * segment_length;
        let segment_end = if i == num_threads - 1 {
            end_index // Last thread takes the remainder
        } else {
            segment_start + segment_length - 1
        };

        let format_code = format_code.clone();
        let thread_url = url.to_string();

        let handle = thread::spawn(move || {
            //let range_command = &segment_start+1.to_string()+":"+&segment_end+1.to_string();
            let range_command = format!("{}:{}", segment_start, segment_end);
            println!("Downloading range: {} to {}", segment_start, segment_end);
            run_command(vec![
                "yt-dlp", "--color", "never", "--write-auto-subs", "--embed-subs",
                "-f", &format_code, "--restrict-filenames", "-c", "--skip-unavailable-fragments",
                "--ignore-errors", "-I", &range_command, "-o", "%(playlist_index)sof%(playlist_count)s-%(title)s.%(ext)s",
                &thread_url,
            ]);
            println!("Download complete for segment {} - {}", segment_start, segment_end);
            tx.send(()).unwrap(); // Signal completion
        });

        handles.push(handle);
    }

    // Drop the sender to close the channel when done
    drop(tx);

    // Wait for all downloads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Optional: Process completed downloads
    //while rx.recv() {
    //    println!("Download completed for a segment.");
    //}
}*/

fn download_playlist(url: &str) {
    println!("Fetching playlist format options...");
    run_command(vec!["yt-dlp", "--color", "never", "-I", "1", "-F", url]);

    let format_code = select_format();
    println!("Selected format: {}. Starting playlist download...", format_code);

    // Get the total video count in the playlist
    let total_videos = get_playlist_count(url);
    println!("Total videos in playlist: {}", total_videos);

    // Get download range from user
    let (start_index, end_index) = get_download_range(total_videos);

    // Create a channel for thread communication
    let (tx, rx) = mpsc::channel();

    // Ask the user for the number of concurrent downloads
    let num_threads: usize = read_input("Enter number of concurrent downloads: ")
        .parse()
        .unwrap_or(1); // Default to 1 if parsing fails

    let segment_length = (end_index - start_index + 1) / num_threads;
    let mut handles = vec![];

    let start_time = Instant::now(); // Start timing

    for i in 0..num_threads {
        let tx = tx.clone();
        let segment_start = start_index + i * segment_length;
        let segment_end = if i == num_threads - 1 {
            end_index // Last thread takes the remainder
        } else {
            segment_start + segment_length - 1
        };

        let format_code = format_code.clone();
        let thread_url = url.to_string();

        let handle = thread::spawn(move || {
            let range_command = format!("{}:{}", segment_start, segment_end);
            println!("Downloading range: {} to {}", segment_start, segment_end);
            
            // Start timing for this thread
            let thread_start_time = Instant::now();
            run_command(vec![
                "yt-dlp", "--color", "never", "--write-auto-subs", "--embed-subs",
                "-f", &format_code, "--restrict-filenames", "-c", "--skip-unavailable-fragments",
                "--ignore-errors", "-I", &range_command, "-o", "%(playlist_index)sof%(n_entries)s-%(title)s.%(ext)s",
                &thread_url,
            ]);
            let thread_duration = thread_start_time.elapsed(); // Calculate thread duration
            tx.send(thread_duration).unwrap(); // Send thread duration back to main thread
        });

        handles.push(handle);
    }

    // Drop the sender to close the channel when done
    drop(tx);

    // Wait for all downloads to complete and collect elapsed times
    let mut total_durations = vec![];
    for _ in handles {
        let duration = rx.recv().expect("Failed to receive duration");
        total_durations.push(duration);
    }

    for (i, duration) in total_durations.iter().enumerate() {
        println!("Segment {} download completed in {:?}", i + 1, duration);
    }

    let total_time = start_time.elapsed(); // Calculate total elapsed time
    println!("Total playlist download complete in {:?}", total_time);
}
//***************

fn get_playlist_count(url: &str) -> usize {
    // Run yt-dlp to get the playlist count
    let output = Command::new("yt-dlp")
        .arg("--color")
        .arg("never")
        .arg("-I")
        .arg("0")
        .arg("-O")
        .arg("playlist:playlist_count")
        .arg(url)
        .output()
        .expect("Failed to execute yt-dlp command");

    let count_str = String::from_utf8_lossy(&output.stdout);
    count_str.trim().parse::<usize>().unwrap_or(0) // Parse to usize or return 0 on failure
}

fn get_download_range(total_videos: usize) -> (usize, usize) {
    println!("Enter start index (0 to {}) (default 0): ", total_videos - 1);
    let start_index: usize = read_input("Start index: ")
        .parse()
        .unwrap_or(0); // Default to 0 if parsing fails

    println!("Enter end index ({} to {}) (default end): ", start_index, total_videos - 1);
    let end_index: usize = read_input("End index: ")
        .parse()
        .unwrap_or(total_videos - 1); // Default to last index if parsing fails

    (start_index, end_index)
}

/*
fn download_video(url: &str) {
    println!("Fetching video format options...");
    run_command(vec!["yt-dlp", "--color", "never", "-F", url]);

    let format_code = select_format();
    println!("Selected format: {}. Starting video download...", format_code);

    run_command(vec![
        "yt-dlp", "--color", "never", "--write-auto-subs", "--embed-subs", "-f", &format_code,
        "--restrict-filenames", "-c", "--skip-unavailable-fragments", "--ignore-errors",
        "-o", "%(title)s.%(ext)s", url,
    ]);
}*/

fn download_video(url: &str) {
    println!("Fetching video format options...");
    run_command(vec!["yt-dlp", "--color", "never", "-F", url]);

    let format_code = select_format();
    println!("Selected format: {}. Starting video download...", format_code);

    let start_time = Instant::now(); // Start timing

    run_command(vec![
        "yt-dlp", "--color", "never", "--write-auto-subs", "--embed-subs", "-f", &format_code,
        "--restrict-filenames", "-c", "--skip-unavailable-fragments", "--ignore-errors",
        "-o", "%(title)s.%(ext)s", url,
    ]);

    let duration = start_time.elapsed(); // Calculate elapsed time
    println!("Video download complete in {:?}", duration);
}

fn select_format() -> String {
    let default_format = "18".to_string();
    println!("Enter format video+audio or press Enter for [{}]: ", default_format);
    let format_input = get_input_with_timeout(10).unwrap_or(default_format.clone());

    let format_code = if format_input.is_empty() {
        default_format
    } else {
        format_input
    };

    println!("Selected format: {}. Confirm? (Y/n): ", format_code);
    let confirmation = get_input_with_timeout(3).unwrap_or("y".to_string()).to_lowercase();

    if confirmation == "n" {
        select_format()  // Retry if user wants to select format again
    } else {
        format_code
    }
}

fn run_command(args: Vec<&str>) {
    let status = Command::new(args[0])
        .args(&args[1..])
        .status()
        .expect("Failed to execute yt-dlp command");

    if !status.success() {
        eprintln!("yt-dlp command failed.");
    }
}

fn read_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string() // Trim the input and convert to String
}

fn get_input_with_timeout(timeout: u64) -> Option<String> {
    let (sender, receiver) = mpsc::channel();

    // Spawn a thread to handle user input
    thread::spawn(move || {
        let mut input = String::new();
        io::stdout().flush().unwrap();  // Ensure prompt is displayed immediately
        if io::stdin().read_line(&mut input).is_ok() {
            let _ = sender.send(input.trim().to_string());
        }
    });

    // Wait for input with a timeout
    receiver.recv_timeout(Duration::from_secs(timeout)).ok()
}
/*
fn download_playlist(url: &str) {
    println!("Fetching playlist format options...");
    run_command(vec!["yt-dlp", "--color", "never", "-I", "1", "-F", url]);

    let format_code = select_format();
    println!("Selected format: {}. Starting playlist download...", format_code);

    // Get the total video count in the playlist
    let total_videos = get_playlist_count(url);
    println!("Total videos in playlist: {}", total_videos);

    // Get download range from user
    let (start_index, end_index) = get_download_range(total_videos);

    // Create an Arc to share format_code across threads
    let format_code_arc = Arc::new(format_code);

    // Create a channel for thread communication
    let (tx, rx) = mpsc::channel();

    // Ask the user for the number of concurrent downloads
    let num_threads: usize = read_input("Enter number of concurrent downloads: ")
        .parse()
        .unwrap_or(1); // Default to 1 if parsing fails

    // Create a pool of worker threads
    let mut handles = vec![];

    for _ in 0..num_threads {
        let tx = tx.clone();
        let format_code = Arc::clone(&format_code_arc);
        let video_indices = (start_index..=end_index).collect::<Vec<_>>();

        let handle = thread::spawn(move || {
            for index in video_indices.iter() {
                //let video_url = format!("{}?index={}", url, index);
                println!("Downloading video at index {}: {}", index, url);
                run_command(vec![
                    "yt-dlp", "--color", "never", "--write-auto-subs", "--embed-subs",
                    "-f", &format_code, "--restrict-filenames", "-c", "--skip-unavailable-fragments",
                    "--ignore-errors", format!("-I {}",index), "-o", "%(playlist_index)s/%(n_entries)s-%(title)s.%(ext)s",
                    &url,
                ]);
                tx.send(index).unwrap(); // Send completion signal
            }
        });

        handles.push(handle);
    }

    // Drop the sender to close the channel when done
    drop(tx);

    // Wait for all downloads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Optional: Process completed downloads
    while let Ok(completed_index) = rx.recv() {
        println!("Completed download for video at index: {}", completed_index);
    }
}*/


