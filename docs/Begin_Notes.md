## Today

------

## Movement features

- !left ALEX 300
- !right ALEX 300
- !up
- !down

------

## TODOs

- quick make certain scenes !flash_sale NAME
- Update hotkeys
- Make beginbotbot, be the same as beginbot in chat

## TODO

- Make skybox chat interface more fun and flexible
- Polls
- Automate Creation of AI Scenes
- More channel points economy features
- How do allow Subs to have default TTS?
  - Without it doubling for Channel Point rewards
- Fix chunking and printing out skybox styles
- Splitting Voice Dreams, split out a single persons voice for easier cloning
- Bring back downloading sounds w/ yt-dlp
- bring back themesongs
- Figure out how to bootstrap the 3 creation of the 6 filters (3 3D Transforms) and (move transitions)!
- Evoling Lore:
  - We can save Chat GPTity responses, and start refeeding for Lore
- Show the the global state3 on stream, with Text
- We need to recombine the 2 audio loops?

## Stream QoL Improvements

- Twitch User Access Token Updating
- Modes to turn off various features
- Figure out getting Messages from Twitch into Neovim
- Way for being to toggle the permissions of every single command

### Choosing Voices Improvements

- Figure out letting command to choose random boy or voice 
- a set voice random that randomly selected a voice for you but assigned it so you could see what it is with the my voice command
- Use ChatGPT to pre-scan the text to determine 1 of x preset categories of music
- Detect language, translate, then read it out
  - Save the language to a file somewhere, and figure out translated
- Permanent way of saving voice transformations
  - !transform_voice
- the ability to add commands mid sentence to change params for specific words
  - Example 1: Hello how are you, I just fell into a !reverb CAAAAAVE
    - We'd had to parse all commands and split audio and restich together
  - Example 2: Hello how are (!reverb,!echo) you
- voice normalization for streamer voices
- Faster way of lower voice, saving in DB
- Don't have emojis be read from Text to Speech
- Get Uberduck features working
  - Hooked up in Rust
  - More Uberduck rapping
- Add an Echo:
  -> Only one word, and try and replicate the 6 effect

### Scenes

- Needledrop Scene
    - Need to clone voice 
        - Need to figure out if we need music?
        - Or make a fake Needledrop scene
- Action Movie
- Finish Melkey Scene
    - https://www.youtube.com/watch?v=pSZKl8dvdcc&list=PLsusUte3YvjAjMQwWBR30sPicwRf9giBJ
- Naurdwar SCENE!!!!
- "Art Critic" , Gen Image from users prompt -> AI describe the image -> feed to art critic prompt -> ElevenLabs etc
- Invent Teej Mode
    - Dalle Prompt dumb dog stupid keyboard
    - Personality Prompt obssesed with jane street shilling languages to the highest bidder
        - Music
- Goodfellas intro style -
  - https://www.youtube.com/watch?v=I7OimVJLzSI

        
### Bugs

- We can't edit sounds, without the voices.json being fucked up
- Track down double-Gippty bug

---

## Art Gallery Improvements

- More Gallery Scenes
- Make sure we can show different peoples curations
- Being able to choose, youtube links or other things for you "curated show"

## Scenes/Jokes

- Figure out an OBS Joke for every single Music scene!
- Teej into the Vortex
  - https://imgur.com/qTNrCCv

---

### Poll Dreams

- How do we dynamically create polls from chat messages:
  - !this hospital drama
  - Then that creates a poll of the options
  - at the end of the poll we know who won
  - and we go to that scene

### Refactor Feelings

We have 3 files:
    - move_transition
    - move_transition_effects
    - stream_fx

This all contains very similiar logic, not sure what the divide should be,
We should maybe recombine, and then clean up and organize

---

## Rust Learnings

https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=db7477cdc022e10152d5b69b3b5b0050

- unwrap_or or unwrap_or_else
- Easier formatting
- More Tests
- Should we create more "service" crates?
- #[deny(elided_lifetimes_in_paths)]

---

### Twitch Specific

- Emotes
    - Animated
    - Regular
- Channel Points Images
- Trailer

---

### API-Explorations

- VALL-E for voice cloning
- https://github.com/facebookresearch/llama/issues/425
- Replicate
- https://www.move.ai/
- https://api-docs.runway.team/#tag/release/operation/updateReleaseById

### Semantic Dimensions

- Filter to all things in the stream:
  - we hit 1800's mode
    - all images generated are more 1800's like
    - the voices have filters on them to make them sound more old-timey
    - all music is more old-timey
    - my scene layout is more old-timey
