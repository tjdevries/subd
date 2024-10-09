# Adding a new AI Scene

#### I: Chat-GPT-content??? (This could be taken through a Gist)

User-supplied:
  - reward_title
  - base_prompt
  - base_dalle_prompt

#### II: Voice (This could be combined into a multi-command)

2 Options:
  - Choose exisiting
  - clone new one
    - Right now you have to clone, then use
    - `!clone NAME URL URL URL`

#### III. Background Music (This can automated, through Twitch Chat asa  standalone, then as a multi-command)

2 Options:
  - Single Song:
    - Save single song to `/home/begin/stream/Stream/BackgroundMusic/SONG_NAME`
  - Playlist:
    - Save playlist to `/home/begin/stream/Stream/BackgroundMusic/PLAYLIST_NAME`

```bash
yt-dlp -f 'ba' -x --audio-format mp3 "YOUTUBE_OR_YOUTUBE_PLAYLIST_URL"
```

```json
{
    "reward_title": "RomCom Trailer",
    "base_prompt": "Romcom voiceover, no stage directions or instructions, just the text. be romantic and funny. no more than 80 words. ",
    "base_dalle_prompt": "Generate a prompt to be used with Dall-E. DO NOT INCLUDE THE word DALLE. Make a RomCom style Movie Poster. Base the poster on the follow information: ",
    "voice": "Fin",
    "music_bg": "!romcom"
},
```

### IV. Create BackgroundMusic Source (Can Automate)

- in OBS on BackgroundMusic scene

### V. Update Subd code and JSON (Can move the subd code to JSON, then automate)

the !begin shoudl be the music_bg command you supply to the json below:
```rust
src/music_scenes.rs

(
    "!begin",
    NewVoiceScene {
        voice: "beginbot",
        music: "Carti-BG-Music",
        playlist_folder: None,
    },
),
```

#### VI. Restart and Test

- have to restart and test!

----

This would be cool!

### Multi-Command

- !ai_scene GIST_OF_JSON, this would also have voice clone links, background music links

