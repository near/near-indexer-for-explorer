CREATE INDEX transactions_actions_action_kind_idx ON transaction_actions (action_kind);
CREATE INDEX action_receipt_actions_action_kind_idx ON action_receipt_actions (action_kind);
CREATE INDEX execution_outcomes_status_idx ON execution_outcomes (status);
