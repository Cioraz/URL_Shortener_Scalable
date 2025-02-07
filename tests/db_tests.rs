use url_shortener::db;

#[cfg(test)]
mod tests {
    use super::*;
    use db::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::task;

    // Initialize a test Redis database connection
    async fn init_test_db() -> Database {
        let client =
            redis::Client::open("redis://127.0.0.1/").expect("Failed to create Redis client");
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");
        Arc::new(Mutex::new(connection))
    }

    // Test concurrent storage and retrieval of data
    #[tokio::test]
    async fn test_concurrent_store_and_retrieve() {
        let db = init_test_db().await;
        let short_url_ids = vec!["test_key1", "test_key2", "test_key3"];
        let long_urls = vec![
            "http://example.com/url1",
            "http://example.com/url2",
            "http://example.com/url3",
        ];

        let mut handles = vec![];

        for (i, &short_url_id) in short_url_ids.iter().enumerate() {
            let db_clone = db.clone();
            let short_url_id = short_url_id.to_string();
            let long_url = long_urls[i].to_string();

            let handle = task::spawn(async move {
                let data = Data {
                    creation_data: chrono::Local::now().to_rfc3339(),
                    shortened_url: format!("http://localhost/{}", short_url_id),
                    long_url,
                    ttl: 30,
                };

                // Store data in Redis
                let store_result =
                    store_data(db_clone.clone(), short_url_id.clone(), data.clone()).await;
                assert!(store_result.is_ok());

                // Retrieve the stored data
                let retrieved_data = retrieve_data(db_clone.clone(), &short_url_id).await;
                assert!(retrieved_data.is_some());

                // Verify that the retrieved data matches the stored data
                if let Some(retrieved) = retrieved_data {
                    assert_eq!(retrieved.shortened_url, data.shortened_url);
                    assert_eq!(retrieved.long_url, data.long_url);
                    assert_eq!(retrieved.ttl, data.ttl);
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    // Test concurrent TTL expiry
    #[tokio::test]
    async fn test_concurrent_ttl() {
        let db = init_test_db().await;
        let short_url_ids = vec!["ttl_key1", "ttl_key2", "ttl_key3"];

        let mut handles = vec![];

        for &short_url_id in &short_url_ids {
            let db_clone = db.clone();
            let short_url_id = short_url_id.to_string();

            let handle = task::spawn(async move {
                let data = Data {
                    creation_data: chrono::Local::now().to_rfc3339(),
                    shortened_url: format!("http://localhost/{}", short_url_id),
                    long_url: "http://example.com/some/long/url".to_string(),
                    ttl: 2,
                };

                // Store data in Redis
                let store_result =
                    store_data(db_clone.clone(), short_url_id.clone(), data.clone()).await;
                assert!(store_result.is_ok());

                // Wait for the TTL to expire
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Verify that the data has been deleted
                let retrieved_after_ttl = retrieve_data(db_clone.clone(), &short_url_id).await;
                assert!(retrieved_after_ttl.is_none());

                // Clean up
                let _ = delete_data(db_clone.clone(), &short_url_id).await;
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    // Test concurrent deletion of data
    #[tokio::test]
    async fn test_concurrent_delete() {
        let db = init_test_db().await;
        let short_url_ids = vec!["delete_key1", "delete_key2", "delete_key3"];

        let mut handles = vec![];

        for &short_url_id in &short_url_ids {
            let db_clone = db.clone();
            let short_url_id = short_url_id.to_string();

            let handle = task::spawn(async move {
                let data = Data {
                    creation_data: chrono::Local::now().to_rfc3339(),
                    shortened_url: format!("http://localhost/{}", short_url_id),
                    long_url: "http://example.com/some/long/url".to_string(),
                    ttl: 30,
                };

                // Store data in Redis
                let store_result =
                    store_data(db_clone.clone(), short_url_id.clone(), data.clone()).await;
                assert!(store_result.is_ok());

                // Delete the stored data
                let delete_result = delete_data(db_clone.clone(), &short_url_id).await;
                assert!(delete_result.is_ok());

                // Verify that the data has been deleted
                let retrieved_after_delete = retrieve_data(db_clone.clone(), &short_url_id).await;
                assert!(retrieved_after_delete.is_none());
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
