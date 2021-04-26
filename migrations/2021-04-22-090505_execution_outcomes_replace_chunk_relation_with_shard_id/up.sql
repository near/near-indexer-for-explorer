-- Setting default value 0 for further fill with data to avoid making the field nullable
ALTER TABLE execution_outcomes ADD COLUMN shard_id numeric(20,0) NOT NULL DEFAULT 0;

UPDATE execution_outcomes SET shard_id = chunks.shard_id FROM chunks WHERE execution_outcomes.executed_in_chunk_hash = chunks.chunk_hash;

ALTER TABLE execution_outcomes ALTER COLUMN shard_id DROP DEFAULT;
ALTER TABLE execution_outcomes DROP COLUMN executed_in_chunk_hash;
