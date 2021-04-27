-- Setting default value to empty string for further fill with data to avoid making the field nullable
ALTER TABLE execution_outcomes ADD COLUMN executed_in_chunk_hash text NOT NULL DEFAULT '';

UPDATE execution_outcomes SET executed_in_chunk_hash = chunks.chunk_hash
    FROM chunks
        WHERE execution_outcomes.executed_in_block_hash = chunks.included_in_block_hash
            AND execution_outcomes.shard_id = chunks.shard_id;

ALTER TABLE execution_outcomes ALTER COLUMN executed_in_chunk_hash DROP DEFAULT;
ALTER TABLE execution_outcomes DROP COLUMN shard_id;
