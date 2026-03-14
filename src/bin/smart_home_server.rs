use std::env;

#[tokio::main]
async fn main() {
    smart_home::logger::init();

    let addr = parse_addr_from_args();

    if let Err(err) = smart_home::server::run_server(addr).await {
        eprintln!("server failed: {err}");
        std::process::exit(1);
    }
}

fn parse_addr_from_args() -> Option<String> {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--addr" {
            return args.next();
        }
    }

    None
}
