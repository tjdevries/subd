resetdb:
	sqlx db drop -y
	sqlx db create
	sqlx migrate run --source crates/subd-db/migrations
