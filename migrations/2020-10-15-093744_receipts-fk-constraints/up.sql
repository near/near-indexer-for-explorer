ALTER TABLE receipt_data
    ADD CONSTRAINT receipt_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE;

ALTER TABLE receipt_actions
    ADD CONSTRAINT receipt_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE;

ALTER TABLE receipt_action_output_data
    ADD CONSTRAINT receipt_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE;

ALTER TABLE receipt_action_input_data
    ADD CONSTRAINT receipt_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE;
