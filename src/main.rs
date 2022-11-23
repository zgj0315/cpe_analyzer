mod cpe;
mod cve;
mod data_stat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .event_format(format)
        .init();
    cpe::download_cpe().await?;
    cpe::put_cpe_to_db().await?;
    cve::download_cve().await?;
    cve::put_cpe_to_db().await?;
    data_stat::cpe_clean().await?;
    data_stat::cpe_stat().await?;
    Ok(())
}
