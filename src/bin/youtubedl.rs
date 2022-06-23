#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = 76;
    let end = 82;

    // --external-downloader ffmpeg --external-downloader-args "ffmpeg_i:-ss 0 -to 10"
    let dl = youtube_dl::YoutubeDl::new("https://youtu.be/bWcASV2sey0")
        .extract_audio(true)
        .extra_arg("--external-downloader")
        .extra_arg("ffmpeg")
        .extra_arg("--external-downloader-args")
        .extra_arg(format!("ffmpeg_i:-ss {} -to {}", start, end))
        .run()?;

    match dl {
        youtube_dl::YoutubeDlOutput::SingleVideo(video) => {
            println!("Video: {:?}", video);
        }
        youtube_dl::YoutubeDlOutput::Playlist(_) => unreachable!(),
    }
    //  https://youtu.be/bWcASV2sey0
    //  76, 6ish
    Ok(())
}
