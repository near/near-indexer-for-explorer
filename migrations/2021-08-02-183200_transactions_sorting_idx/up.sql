CREATE INDEX CONCURRENTLY transactions_sorting_idx ON transactions (block_timestamp, index_in_chunk);
