use rsky_pds::build_rocket;
use std::fmt::Debug;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[rocket::main]
async fn main() {
    // let filter = filter::Targets::new()
    //     // Enable the `INFO` level for anything in `my_crate`
    //     .with_target("my_crate", Level::INFO)
    //     // Enable the `DEBUG` level for a specific module.
    //     .with_target("my_crate::interesting_module", Level::DEBUG);
    //
    // // Build a new subscriber with the `fmt` layer using the `Targets`
    // // filter we constructed above.
    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::fmt::layer())
    //     .with(filter)
    //     .init();

    let subscriber = tracing_subscriber::FmtSubscriber::new();
    // let subscriber2 = tracing_subscriber::fmt()
    //     .compact()
    //     .with_file(true)
    //     .with_line_number(true)
    //     .with_thread_ids(true)
    //     .with_target(true)
    //     .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let _ = build_rocket(None).await.launch().await;
}
