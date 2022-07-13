default:
	cargo build

chat default:
	cargo run --bin chat

resetdb:
	sqlx db drop -y
	sqlx db create
	sqlx migrate run --source crates/subd-db/migrations

serve:
	trunk serve

release:
	trunk build --release.

# -c clear
# -x execute
watch:
	cargo watch -c -x 'run --bin chat'

install-trunk:
	cargo install --locked trunk
	cargo install wasm-bindgen-cli
