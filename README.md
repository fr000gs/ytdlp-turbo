# ytdlp-turbo

`ytdlp-turbo` is a command-line tool for downloading videos and playlists.

## Features

- **Playlist Support**: Download entire playlists or specific ranges of videos.
- **Multithreading**: Speed up downloads with concurrent threads.

## Requirements

- [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) (ensure it is installed and accessible via the command line)(install via pip bcz I did it so)
- [`ffmpeg`](https://www.ffmpeg.org/download.html)(to mix video+audio & subtitles)

## Examples

### Download a Playlist

```bash
./ytdlp-turbo
```
Provide the playlist URL, specify the range (e.g., videos 1-10), and configure the number of threads.

## Contributing

Contributions are welcome! If you have ideas, bug reports, or improvements, feel free to open an issue or submit a pull request.

