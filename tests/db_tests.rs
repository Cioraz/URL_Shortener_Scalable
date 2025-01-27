use url_shortener::db;

#[cfg(test)]
mod tests {
    use super::*;
    use db::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

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

    // #[tokio::test]
    // async fn test_store_data() {
    //     let db = init_test_db().await;
    //     let short_url_id = "test_key".to_string();
    //     let test_time = "dummy time";
    //     let data = Data {
    //         creation_data: test_time.to_string(),
    //         shortened_url: "http://localhost/test_short".to_string(),
    //         long_url: "http://example.com/some/long/url".to_string(),
    //         ttl: 30,
    //     };

    //     // Store data in Redis
    //     let result = store_data(db.clone(), short_url_id.clone(), data.clone()).await;
    //     assert!(result.is_ok());

    //     // Retrieve the stored data
    //     let retrieved_data = retrieve_data(db.clone(), &short_url_id).await;
    //     assert!(retrieved_data.is_some());

    //     // Verify that the retrieved data matches the stored data but creation data changes so not testing that
    //     if let Some(retrieved) = retrieved_data {
    //         assert_eq!(retrieved.shortened_url, data.shortened_url);
    //         assert_eq!(retrieved.long_url, data.long_url);
    //         assert_eq!(retrieved.ttl, data.ttl);
    //     }
    // }

    // Test if TTL is working as expected
    #[tokio::test]
    async fn test_ttl() {
        let db = init_test_db().await;
        let short_url_id = "test_key".to_string();
        let data = Data {
            creation_data: chrono::Local::now().to_rfc3339(),
            shortened_url: "http://localhost/test_short".to_string(),
            long_url: "http://example.com/some/long/url".to_string(),
            ttl: 2,
        };

        // Store data in Redis
        let result = store_data(db.clone(), short_url_id.clone(), data.clone()).await;
        assert!(result.is_ok());

        // Retrieve the stored data
        let retrieved_data = retrieve_data(db.clone(), &short_url_id).await;
        assert!(retrieved_data.is_some());

        // Wait for the TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Verify that the data has been deleted
        let retrieved_after_ttl = retrieve_data(db.clone(), &short_url_id).await;
        assert!(retrieved_after_ttl.is_none());

        // remove the data from the database dont return it
        match delete_data(db.clone(), &short_url_id).await {
            Ok(_) => (),
            Err(e) => panic!("Error deleting data: {}", e),
        }
    }

    // Test if the data is being retrieved correctly
    #[tokio::test]
    async fn test_retrieve_nonexistent_data() {
        let db = init_test_db().await;
        let short_url_id = "nonexistent_key";

        // Attempt to retrieve nonexistent data
        let retrieved_data = retrieve_data(db.clone(), short_url_id).await;
        assert!(retrieved_data.is_none());
    }

    // Test if data is being deleted correctly
    #[tokio::test]
    async fn test_delete_data() {
        let db = init_test_db().await;
        let short_url_id = "delete_key".to_string();

        // Prepare and store data
        let data = Data {
            creation_data: chrono::Local::now().to_rfc3339(),
            shortened_url: "http://localhost/test_short".to_string(),
            long_url: "http://example.com/some/long/url".to_string(),
            ttl: 30,
        };

        store_data(db.clone(), short_url_id.clone(), data)
            .await
            .unwrap();

        // Delete the stored data
        let delete_result = delete_data(db.clone(), &short_url_id).await;
        assert!(delete_result.is_ok());

        // Verify that the data has been deleted
        let retrieved_after_delete = retrieve_data(db.clone(), &short_url_id).await;
        assert!(retrieved_after_delete.is_none());
    }
}
