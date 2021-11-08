#[macro_export]
macro_rules! await_retry_or_panic {
    ($query: expr, $number_of_retries: expr, $error_message: expr, $debug_structs: expr $(, $is_error_handled:expr)? $(,)?) => {
        {
            let mut interval = crate::INTERVAL;
            let mut retry_attempt = 0usize;
            loop {
                if retry_attempt == $number_of_retries {
                    return Err(
                        anyhow::anyhow!(
                            "Failed to perform query to database after {} attempts. Stop trying.",
                            $number_of_retries
                        )
                    );
                }
                retry_attempt += 1;

                match $query.await {
                    Ok(res) => break Some(res),
                    Err(async_error) => {
                        $(if $is_error_handled(&async_error).await {
                            break None;
                        })?

                        tracing::error!(
                             target: crate::INDEXER_FOR_EXPLORER,
                             "Error occurred during {}: \n{:#?} \n{:#?} \n Retrying in {} milliseconds...",
                             async_error,
                             &$error_message,
                             &$debug_structs,
                             interval.as_millis(),
                         );
                        tokio::time::sleep(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };
}
