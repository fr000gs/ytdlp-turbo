use std::io::{self, Write};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn main() -> ! {
    loop {
        let use_playlist_format = read_input("Same format for playlist? [Y/n]: ")
            .parse()
            .unwrap_or("y".to_string())
            .to_lowercase();

        let is_playlist = to_boolean(&use_playlist_format);

        //let is_playlist = use_playlist_format == "y";

        let url = read_input("Enter URL: ");

        let separate_folders_foreach = to_boolean(
            &read_input("Make separate folders for each video?[Y/n]")
                .parse()
                .unwrap_or("y".to_string())
                .to_lowercase()
                .as_str(),
        );

        if is_playlist {
            download_playlist(&url, separate_folders_foreach);
        } else {
            download_video(&url);
        }
    }
}

fn ask_if_format() -> bool {
    to_boolean(
        &read_input("Display formats? [Y/n]: ")
            .parse()
            .unwrap_or("y".to_string())
            .to_lowercase(),
    )
}

fn to_boolean(input: &str) -> bool {
    match input.trim().to_lowercase().as_str() {
        "y" => true,
        "n" => false,
        _ => false,
    }
}

fn download_playlist(url: &str, separate_folder: bool) {
    if ask_if_format() {
        println!("Fetching playlist format options...");
        run_command(vec!["yt-dlp", "--color", "never", "-I", "1", "-F", url])
    }

    let format_code = select_format();
    println!(
        "Selected format: {}. Starting playlist download...",
        format_code
    );

    let total_videos = get_playlist_count(url);
    println!("Total videos in playlist: {}", total_videos);

    let (start_index, end_index) = get_download_range(total_videos);

    // Create a channel for thread communication
    let (tx, rx) = mpsc::channel();

    // Ask the user for the number of concurrent downloads
    let num_threads: usize = read_input("Enter number of concurrent downloads: ")
        .parse()
        .unwrap_or(total_videos / 2);

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
                "yt-dlp",
                "--color",
                "never",
                "--write-auto-subs",
                "--embed-subs",
                "-f",
                &format_code,
                "--restrict-filenames",
                "-c",
                "--skip-unavailable-fragments",
                "--ignore-errors",
                "-I",
                &range_command,
                "-o",
                if separate_folder {
                    "%(playlist_index)sof%(playlist_count)s-%(title)s/%(playlist_index)sof%(playlist_count)s-%(title)s.%(ext)s"
                } else {
                    "%(playlist_index)sof%(playlist_count)s-%(title)s.%(ext)s"
                },
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

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0)
}

fn get_download_range(total_videos: usize) -> (usize, usize) {
    println!(
        "Enter start index (0 to {}) (default 0): ",
        total_videos - 1
    );
    let start_index: usize = read_input("Start index: ").parse().unwrap_or(0);

    println!(
        "Enter end index ({} to {}) (default end): ",
        start_index,
        total_videos - 1
    );
    let end_index: usize = read_input("End index: ")
        .parse()
        .unwrap_or(total_videos - 1);

    (start_index, end_index)
}

fn download_video(url: &str) {
    if ask_if_format() {
        println!("Fetching video format options...");
        run_command(vec!["yt-dlp", "--color", "never", "-F", url]);
    }

    let format_code = select_format();
    println!(
        "Selected format: {}. Starting video download...",
        format_code
    );

    let start_time = Instant::now(); // Start timing

    run_command(vec![
        "yt-dlp",
        "--color",
        "never",
        "--write-auto-subs",
        "--embed-subs",
        "-f",
        &format_code,
        "--restrict-filenames",
        "-c",
        "--skip-unavailable-fragments",
        "--ignore-errors",
        "-o",
        "%(title)s.%(ext)s",
        url,
    ]);

    println!("Video download complete in {:?}", start_time.elapsed());
}

fn select_format() -> String {
    let default_format = "18".to_string();
    println!("Default format is 18(360p)");
    let format_input = read_input("Enter format video+audio or press Enter for default format")
        .parse()
        .unwrap_or(default_format.clone());

    let format_code = if format_input.is_empty() {
        default_format.clone()
    } else {
        format_input
    };

    let confirmation =
        read_input(&("Selected format:".to_owned() + &format_code.clone() + ". Confirm? (Y/n): "))
            .parse()
            .unwrap_or("y".to_string())
            .to_lowercase();

    if confirmation == "n" {
        select_format()
    } else {
        format_code
    }
}

fn run_command(args: Vec<&str>) {
    let status = Command::new(args[0])
        .args(&args[1..])
        .status()
        .expect(format!("Failed to execute yt-dlp command{:#?}", &args).as_str());

    if !status.success() {
        eprintln!("{:#?} failed", args);
        eprintln!()
    }
}

fn read_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string() // Trim the input and convert to String
}
