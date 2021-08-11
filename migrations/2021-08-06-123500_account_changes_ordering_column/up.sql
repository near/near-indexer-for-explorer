-- Setting default value -1 for further fill with data to avoid making the field nullable
ALTER TABLE account_changes
    ADD COLUMN index_in_block integer NOT NULL DEFAULT -1;

-- This migration is heavy to apply. Consider using Python script to do that in background.
-- It applies the changes by tiny pieces.
-- You need to replace `END` and `connection_string` parameters

-- from sqlalchemy import create_engine
--
-- START = 1595370903490523743
-- STEP = 1000 * 1000 * 1000 * 1000  # 1000 secs -> ~17 minutes
-- END = 1628683241000000000
-- ESTIMATED_STEPS = (END - START) / STEP
-- connection_string = 'postgresql+psycopg2://user:pass@host/database'
--
--
-- def generate_sql(from_timestamp: int, to_timestamp: int) -> str:
--     """
--     Generates str for SQL query to convert args_base64 to args_json if possible
--     """
--     return f"""
--     BEGIN;
--     WITH indexes AS
--          (
--              SELECT id, row_number() OVER (PARTITION BY changed_in_block_timestamp ORDER BY id) - 1 as index_in_block
--              FROM account_changes
--              WHERE account_changes.changed_in_block_timestamp >= {from_timestamp}
--                 AND account_changes.changed_in_block_timestamp <= {to_timestamp}
--          )
--     UPDATE account_changes
--     SET index_in_block = indexes.index_in_block
--     FROM indexes
--     WHERE account_changes.id = indexes.id
--         AND account_changes.index_in_block = -1
--         AND account_changes.changed_in_block_timestamp >= {from_timestamp}
--         AND account_changes.changed_in_block_timestamp <= {to_timestamp};
--     COMMIT;
--     """
--
--
-- if __name__ == '__main__':
--     print("Establishing connection to %s..." % (connection_string.split('/')[-1],))
--     engine = create_engine(connection_string)
--     print(f"Estimated queries to execute: {ESTIMATED_STEPS}.")
--     from_timestamp = START-STEP
--     to_timestamp = START
--     counter = 1
--
--     with engine.connect() as con:
--         while True:
--             from_timestamp += STEP
--             to_timestamp += STEP
--             if (END - to_timestamp) < STEP:
--                 break
--             print(f"{counter}/{ESTIMATED_STEPS} (from {from_timestamp} to {to_timestamp})")
--
--             out = con.execute(generate_sql(from_timestamp, to_timestamp))
--             counter += 1
--
--     print("FINISHED")

ALTER TABLE account_changes
    ALTER COLUMN index_in_block DROP DEFAULT;
