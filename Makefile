default:
	RUST_BACKTRACE=1 cargo build --bin begin
	# cargo build
	# cargo check

test:
	cargo test

fix:
	cargo fix --lib -p server

chat:
	cargo run --bin chat
	
tts:
	cargo run --bin begin -- --enable tts twitch_chat_saving implict_soundeffects
	# cargo run --bin begin -- --enable tts --enable twitch_chat_saving implict_soundeffects

screenshots_timer:
	cargo run --bin begin -- --enable ai_screenshots_timer

small:
	cargo run --bin begin -- --enable ai_scenes twitch_chat_saving

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
