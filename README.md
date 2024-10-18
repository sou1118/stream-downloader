# Stream Downloader

[![CI](https://github.com/sou1118/stream-downloader/actions/workflows/ci.yml/badge.svg)](https://github.com/sou1118/stream-downloader/actions/workflows/ci.yml)

Stream Downloader is a Rust program designed to download stream episodes from a specific streaming service and save them as MP3 files.

## Features

- Fetches a list of radio episodes from a specified URL
- Parses m3u8 playlists for each episode
- Downloads and decrypts encrypted audio segments
- Converts downloaded audio to MP3 format

## Prerequisites

This program requires the following to be installed on your system:

- Rust programming language
- ffmpeg (for audio conversion)

## Dependencies

The program relies on several external libraries:

- `reqwest`: For sending HTTP requests
- `serde_json`: For parsing JSON data
- `m3u8_rs`: For parsing m3u8 playlists
- `aes` and `cbc`: For AES-CBC decryption
- `anyhow`: For error handling
- `tokio`: For asynchronous runtime

## Usage

1. Ensure you have Rust and ffmpeg installed on your system.
2. Clone or download this project.
3. Open the `main.rs` file and set the `url` variable to the appropriate API endpoint.
4. In the project directory, run the following command:

   ```
   cargo run
   ```

5. Downloaded files will be saved in the `downloads` directory.

## How it works

1. The program fetches episode information from the specified URL.
2. For each episode, it downloads the m3u8 playlist.
3. If the playlist is a master playlist, it selects the first variant.
4. It then downloads and decrypts each audio segment.
5. Finally, it uses ffmpeg to convert the audio to MP3 format.

## Error Handling

The program uses the `anyhow` crate for error handling. If an error occurs during the download of an episode, it will print an error message and continue with the next episode.

## Customization

You can modify the following aspects of the program:

- Output directory: Change the `output_dir` variable in the `download_episode` function.
- MP3 bitrate: Modify the `-b:a` parameter in the ffmpeg command within the `download_media_playlist` function.

## Legal Disclaimer

This program is for educational purposes only. Ensure you have the right to download and store the content. Always comply with the terms of service of the content provider and respect copyright laws.

## License

This project is open-source and available under the Apache License 2.0.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
