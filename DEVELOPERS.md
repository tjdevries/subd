# Developers

## Getting Started

Make sure you have Rust installed
    - https://www.rust-lang.org/tools/install

Make sure you are using the nightly rustc compiler

```
rustup install nightly

rustup override set nightly
```

## Creating and Connecting to Database

- Install Sqlite
- Create Database
- Set DATABASE_URL

```
sqlite subd
```

You need to set the database URL environment variable to connect to the database
For convenience, you can use a .env file to set DATABASE_URL so that you don't have to pass it every time:

```.env
DATABASE_URL="sqlite:///home/user/folder/project/subd.db"
```

You need to install sqlx-cli to build the database

```
cargo build sqlx-cli
```


## Build/Reset Database

```
make resetdb
```

At this point you should be ready to compile

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

TWITCH_BROADCASTER_USERNAME=
TWITCH_BROADCASTER_CHANNEL_ID=
TWITCH_BOT_USERNAME=
TWITCH_BOT_CHANNEL_ID=
```

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
