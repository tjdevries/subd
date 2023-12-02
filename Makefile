default:
	RUST_BACKTRACE=1 cargo build --bin begin
	# cargo build
	# cargo check

fix:
	cargo fix --lib -p server

chat:
	cargo run --bin chat

begin:
	cargo run --bin begin

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
