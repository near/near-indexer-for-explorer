ALTER TABLE receipt_data
    DROP CONSTRAINT receipt_fk;

ALTER TABLE receipt_actions
    DROP CONSTRAINT receipt_fk;

ALTER TABLE receipt_action_output_data
    DROP CONSTRAINT receipt_fk;

ALTER TABLE receipt_action_input_data
    DROP CONSTRAINT receipt_fk;
