# Developers

## Current Begin Refactor

- There is a chance that the program is going to crash everywhere!!!
    - But we just need to make sure the top level can handle the error fine

- clean up
    - obs_source.rs
    - obs_combo.rs
    - stream_character.rs
    - streamfx.rs
    - uberduck.rs

- move_transition
- move_transition_effects
- move_transition_bootstrap

- bootstrap.rs
    - This is an interesting idea for starting my stream from scratch

---

## Next TODOs

- Transform-Text for move-transition struct
    with a Test!!!!!

- figure out colors for outline handler

- Figure out changing states and showing the chat the current state easily

- Clean up individual effects

- Abstract out the filter_type number everywhere

---

## Bootstrap Idea: From Now On

- We start w/ Cam, Desktop, Mic
- We bootstrap from there every stream

---------------------------------------------------

This program connects to many different services, and some of those sources need
special software installed. So it's important to read careful about how to
connect (or ignore those features) to each source.

## Getting Started

Make sure you have Rust installed
    - https://www.rust-lang.org/tools/install

Make sure you are using the nightly rustc compiler

```
rustup install nightly

rustup override set nightly
```

## Compiling


```
cargo build
```

## Running our Programs

```
cargo run --bin chat
```

## Set the Twitch OAUTH Env var

Set this in the .env file!

```
SUBD_TWITCH_BROADCASTER_OAUTH=
SUBD_TWITCH_BOT_OAUTH=
SUBD_GITHUB_TOKEN=

SUDD_TWITCH_BROADCASTER_USERNAME=
SUDD_TWITCH_BROADCASTER_CHANNEL_ID=
SUDD_TWITCH_BOT_USERNAME=
SUDD_TWITCH_BOT_CHANNEL_ID=
```

## Setting Up Yew and Trunk

https://yew.rs/docs/getting-started/project-setup/using-trunk

```
cargo install --locked trunk

cargo install wasm-bindgen-cli

cargo install cargo-watch

rustup target add wasm32-unknown-unknown
```

```
cd crates/subd-yew
```

## Setting Up OBS

If you want to take full advantage of all OBS features, you need to have a
couple Plugins installed.

- [https://github.com/Xaymar/obs-StreamFX](StreamFX)
- [https://github.com/exeldro/obs-move-transition](Move-Transition)

Once you have these installed, you need to make sure your sources have the
proper filters created.

```
!create_filters_for_source INSERT_SOURCE_NAME
```

This will create a Number of Filters:

- Blur
- Scroll
- 3D Transform
- SDF Effects
- Move-Value Filters for each of those move
- Move-Value to Defaults filters
- Move Source on "Primary" scene

These filters will allow your chat to control your scroll, blur, and total 3D
transformation of all you sources!

---

# SOON TO BE DEPRECATED

## Creating and Connecting to Database

Install Postgresql

```
createdb subd
```

```
sqlx db reset
```

## Build/Reset Database

```
make resetdb
```

At this point you should be ready to compile


----

## Architecture

- begin.rs
    - entry point to running a begin stream

## begin.rs

- Create various connections
- Launchs the Event loop with the various handlers
