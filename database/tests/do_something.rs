use diesel::Connection;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

fn run_migrations(database_url: &str) {
    let conn = diesel::PgConnection::establish(database_url).expect("Error connecting to database");
    diesel_migrations::run_pending_migrations(&conn).unwrap();
}

#[actix_rt::test]
async fn do_something() {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").unwrap();

    // run_migrations(&database_url);

    let pool = explorer_database::models::establish_connection(&database_url);

    let dir = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{}/tests/blocks/82699904.json", dir);
    print!("{file_path}");
    let binding = std::fs::read_to_string(file_path).unwrap();
    let block_bytes = binding.as_bytes();

    let block =
        serde_json::from_slice::<near_indexer_primitives::views::BlockView>(block_bytes).unwrap();

    explorer_database::adapters::blocks::store_block(&pool, &block)
        .await
        .unwrap();

    let lastest_block_height = explorer_database::adapters::blocks::latest_block_height(&pool)
        .await
        .unwrap();

    let block_at_timestamp =
        explorer_database::adapters::blocks::get_latest_block_before_timestamp(
            &pool,
            1673432911606790912,
        )
        .await
        .unwrap();

    assert_eq!(lastest_block_height, Some(block.header.height));
    assert_eq!(
        block_at_timestamp.block_height,
        bigdecimal::BigDecimal::from(block.header.height)
    );
}
