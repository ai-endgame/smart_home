use std::env;

#[tokio::main]
async fn main() {
    // Load .env before anything else so DATABASE_URL and other vars are visible.
    dotenvy::dotenv().ok();

    smart_home::logger::init();

    let addr = arg_value("--addr");
    let database_url = arg_value("--database-url")
        .or_else(|| env::var("DATABASE_URL").ok());

    if let Err(err) = smart_home::server::run_server_full(addr, database_url).await {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}

fn arg_value(flag: &str) -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }
    None
}
