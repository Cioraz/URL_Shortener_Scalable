use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use tokio::task;
use url_shortener::db::{self, Data, Pool};

async fn init_test_db() -> Pool {
    db::init_db().await
}

async fn generate_random_string() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..6).map(|_| rng.gen_range(b'a'..b'z') as char).collect();
    chars.into_iter().collect()
}

async fn benchmark_store_and_retrieve(pool: Pool) {
    let mut handles = vec![];
    let collisions = std::sync::Arc::new(tokio::sync::Mutex::new(0));

    for _ in 0..10_000 {
        let pool_clone = pool.clone();
        let collisions = collisions.clone();
        let short_url_id = generate_random_string().await;
        let long_url = format!("http://example.com/{}", generate_random_string().await);

        let handle = task::spawn(async move {
            let data = Data {
                creation_data: chrono::Local::now().to_rfc3339(),
                shortened_url: format!("http://localhost/{}", short_url_id),
                long_url,
                ttl: 30,
            };

            // Store data in Redis
            let store_result =
                db::store_data(&pool_clone, short_url_id.clone(), data.clone()).await;
            assert!(store_result.is_ok());

            // Retrieve the stored data
            let retrieved_data = db::retrieve_data(&pool_clone, &short_url_id).await;
            assert!(retrieved_data.is_some());

            // Check for collisions
            if let Some(retrieved) = retrieved_data {
                if retrieved.shortened_url != data.shortened_url
                    || retrieved.long_url != data.long_url
                {
                    let mut collision_count = collisions.lock().await;
                    *collision_count += 1;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Report collisions
    let final_collisions = *collisions.lock().await;
    if final_collisions > 0 {
        println!("Collisions occurred: {}", final_collisions);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("store_and_retrieve_10000_clients", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool = init_test_db().await;
                benchmark_store_and_retrieve(pool).await
            })
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
