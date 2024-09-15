default:
	RUST_BACKTRACE=1 cargo build --bin begin
	
good:
	RUSTFLAGS=-Awarnings cargo build --bin begin
	
ff:
	cargo test test_fal -- --nocapture
	
ft:
	cargo test test_transform_filters -- --nocapture

t:
	cargo test test_remove_old_bogans -- --nocapture
	
loudtest:
	cargo test -- --nocapture

test:
	cargo test

fix:
	cargo fix --lib -p server

chat:
	cargo run --bin chat
	
sfx:
	cargo run --bin begin -- --enable explicit_soundeffects
	twitch_chat_saving implict_soundeffects
	
tts:
	cargo run --bin begin -- --enable tts twitch_chat_saving implict_soundeffects
	# cargo run --bin begin -- --enable tts twitch_chat_saving implict_soundeffects explicit_soundeffects
	# cargo run --bin begin -- --enable tts --enable twitch_chat_saving implict_soundeffects

screenshots_timer:
	cargo run --bin begin -- --enable ai_screenshots_timer

            # "implict_soundeffects".to_string(),
            # "explicit_soundeffects".to_string(),
            # "tts".to_string(),
            # "ai_screenshots".to_string(),
            # "ai_screenshots_timer".to_string(),
            # "ai_telephone".to_string(),
            # "ai_scenes".to_string(),
            # "skybox".to_string(),
            # "obs".to_string(),
            # "twitch_chat_saving".to_string(),
            # "stream_character".to_string(),
            # "chat_gpt_response".to_string(),
            # "twitch_eventsub".to_string(),
            # "dynamic_stream_background".to_string(),
medium:
	cargo run --bin begin -- --enable \
		ai_scenes \
		twitch_chat_saving \
		dynamic_stream_background \
		twitch_eventsub \
		chat_gpt_response \
		obs \
		skybox \
		ai_scenes \
		ai_telephone \
		ai_screenshots \
		explicit_soundeffects \
		implict_soundeffects

all:
	cargo run --bin begin -- --enable-all

small:
	cargo run --bin begin -- --enable ai_scenes twitch_chat_saving twitch_eventsub obs explicit_soundeffects channel_rewards ai_songs turbo_bg

begin:
	cargo run --bin begin -- --enable-all

resetdb:
	sqlx db drop -y
	sqlx db create
	sqlx migrate run --source migrations
	# sqlx migrate run --source crates/subd-db/migrations

serve:
	cd crates/subd-yew && trunk serve --address 0.0.0.0

release:
	trunk build --release.

# -c clear
# -x execute
watch:
	cargo watch -c -x 'run --bin chat'

install-trunk:
	cargo install --locked trunk
	cargo install wasm-bindgen-cli

error:
	cargo rustc --bin begin -- -Awarnings

fmt:
	cargo fmt --all
