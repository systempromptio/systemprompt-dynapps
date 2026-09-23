-- Seed each feedback workstream after installation; reinstall must preserve it.
INSERT INTO managed_consumer_credentials(device_id,credential_digest) VALUES('feedback-device',repeat('1',64));
INSERT INTO managed_consumer_grants(owner_id,resource_id,consumer_id)
 VALUES('feedback-owner','feedback-resource','feedback-consumer');
INSERT INTO managed_installation_receipts(id,owner_id,installation_id,publication_id,resource_id,generation,bundle_digest,installed_manifest,client_evidence,consumer_id,device_id,host,consumer_evidence,fully_verified)
 VALUES('feedback-receipt','feedback-owner','feedback-native-install','feedback-publication','feedback-resource',1,repeat('c',64),'{}','{}','feedback-consumer','feedback-device','codex','{"readback":"unavailable"}',false);
INSERT INTO managed_consumer_session_bindings(id,receipt_id,consumer_id,device_id,host,native_session_id)
 VALUES('feedback-binding','feedback-receipt','feedback-consumer','feedback-device','codex','native-session');
INSERT INTO managed_inventory_state(owner_id,generation,entries) VALUES('feedback-owner',9,1);
INSERT INTO managed_inventory_entries(owner_id,entry_id,kind,resource_key,origin,resource_id,availability,first_observed_at,last_observed_at,generation)
 VALUES('feedback-owner','feedback-entry','skill','example','imported','feedback-resource','unavailable','2020-01-01T00:00:00Z','2020-02-01T00:00:00Z',9);
INSERT INTO managed_inventory_membership(owner_id,entry_id,effective_from,effective_until,record)
 VALUES('feedback-owner','feedback-entry','2020-01-01T00:00:00Z','2020-02-01T00:00:00Z','{"availability":"available"}'),
 ('feedback-owner','feedback-entry','2020-02-01T00:00:00Z',NULL,'{"availability":"unavailable","prior_membership":"unknown"}');
INSERT INTO managed_inventory_bindings(owner_id,entry_id,resource_id,bound_by)
 VALUES('feedback-owner','feedback-entry','feedback-resource','feedback-owner');
INSERT INTO managed_installation_coverage_state(owner_id,generation) VALUES('feedback-owner',4);
INSERT INTO managed_installation_coverage(owner_id,resource_id,body,generation)
 VALUES('feedback-owner','feedback-resource','{"eligible_devices":1,"current_verified_devices":0}',4);
INSERT INTO plugin_usage_events(id,user_id,session_id,event_type,prompt_preview,created_at)
 VALUES('feedback-raw','feedback-owner','owner-session','UserPromptSubmit','/fixture:example','2020-01-01T00:00:00Z');

-- AI024 changes only the failure projection; settled usage/cost/time stay retained.
UPDATE ai_requests SET status='failed',accounting_failed_at='2020-01-02T00:00:00Z',accounting_error='fixture accounting mismatch',error_message='fixture accounting mismatch' WHERE id='feedback-ai-completed';
