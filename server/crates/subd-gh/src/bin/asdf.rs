use subd_gh::is_user_sponsoring;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello world");

    is_user_sponsoring("jesseleite").await?;

    Ok(())
}
