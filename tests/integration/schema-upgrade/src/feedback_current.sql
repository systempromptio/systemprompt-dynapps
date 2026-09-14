-- Seed each feedback workstream after installation; reinstall must preserve it.
INSERT INTO managed_consumer_credentials(device_id,credential_digest) VALUES('feedback-device',repeat('1',64));
INSERT INTO managed_consumer_grants(owner_id,resource_id,consumer_id)
 VALUES('feedback-owner','feedback-resource','feedback-consumer');
INSERT INTO managed_installation_receipts(id,owner_id,installation_id,publication_id,resource_id,generation,bundle_digest,installed_manifest,client_evidence,consumer_id,device_id,host,consumer_evidence,fully_verified)
 VALUES('feedback-receipt','feedback-owner','feedback-native-install','feedback-publication','feedback-resource',1,repeat('c',64),'{}','{}','feedback-consumer','feedback-device','codex','{"readback":"unavailable"}',false);
INSERT INTO managed_consumer_session_bindings(id,receipt_id,consumer_id,device_id,host,native_session_id)
 VALUES('feedback-binding','feedback-receipt','feedback-consumer','feedback-device','codex','native-session');
INSERT INTO managed_consumer_invocation_evidence(id,consumer_id,device_id,host,native_session_id,invocation_id,resource_id,evidence,occurred_at)
 VALUES('feedback-evidence','feedback-consumer','feedback-device','codex','native-session','native-invocation','feedback-resource','{"original":"immutable"}','2020-01-01T00:00:00Z');
INSERT INTO managed_consumer_attribution_projection(evidence_id,receipt_id,version)
 VALUES('feedback-evidence','feedback-receipt',2);
INSERT INTO managed_consumer_attribution_history(evidence_id,version,receipt_id,reason)
 VALUES('feedback-evidence',1,NULL,'unknown'),('feedback-evidence',2,'feedback-receipt','late receipt');
INSERT INTO managed_dependency_verifications(id,owner_id,revision_id,commit_sha,bundle_digest,manifest)
 VALUES('feedback-verification','feedback-owner','feedback-revision',repeat('a',40),repeat('c',64),'{"version":1,"modes_verified":true,"dependencies":["own-source"]}');
INSERT INTO managed_resource_git_bindings(owner_id,resource_id,source_id,relative_root,bound_by)
 VALUES('feedback-owner','feedback-resource','feedback-source','skills/example','feedback-owner');
INSERT INTO managed_inventory_state(owner_id,generation,entries) VALUES('feedback-owner',9,1);
INSERT INTO managed_inventory_entries(owner_id,entry_id,kind,resource_key,origin,resource_id,availability,first_observed_at,last_observed_at,generation)
 VALUES('feedback-owner','feedback-entry','skill','example','imported','feedback-resource','unavailable','2020-01-01T00:00:00Z','2020-02-01T00:00:00Z',9);
INSERT INTO managed_inventory_membership(owner_id,entry_id,effective_from,effective_until,record)
 VALUES('feedback-owner','feedback-entry','2020-01-01T00:00:00Z','2020-02-01T00:00:00Z','{"availability":"available"}'),
 ('feedback-owner','feedback-entry','2020-02-01T00:00:00Z',NULL,'{"availability":"unavailable","prior_membership":"unknown"}');
INSERT INTO managed_inventory_bindings(owner_id,entry_id,resource_id,bound_by)
 VALUES('feedback-owner','feedback-entry','feedback-resource','feedback-owner');
INSERT INTO managed_inventory_captures(owner_id,entry_id,operation_id,status,revision_id)
 VALUES('feedback-owner','feedback-entry','feedback-capture','complete','feedback-revision');
INSERT INTO managed_installation_coverage_state(owner_id,generation) VALUES('feedback-owner',4);
INSERT INTO managed_installation_coverage(owner_id,resource_id,body,generation)
 VALUES('feedback-owner','feedback-resource','{"eligible_devices":1,"current_verified_devices":0}',4);
INSERT INTO analytics_fact_checkpoints(owner_id,generation,applied_changes) VALUES('feedback-owner',3,3);
INSERT INTO analytics_fact_changes(owner_id,change_id,fact_kind,source,fact_id,revision,occurred_at,recorded_at,payload,payload_digest,state,lease_epoch)
 VALUES('feedback-owner','feedback-change','request','fixture','request',2,'2020-01-01T00:00:00Z','2020-01-02T00:00:00Z','{"corrected":true}',repeat('2',64),'applied',7);
INSERT INTO analytics_normalized_facts(owner_id,fact_kind,source,fact_id,revision,occurred_at,deleted,fact,generation)
 VALUES('feedback-owner','request','fixture','request',2,'2020-01-01T00:00:00Z',true,NULL,3);
INSERT INTO analytics_fact_deltas(owner_id,generation,fact_kind,source,fact_id,occurred_at,before_fact,after_fact,consumed_at)
 VALUES('feedback-owner',3,'request','fixture','request','2020-01-01T00:00:00Z','{"amount":5}',NULL,now());
INSERT INTO analytics_fact_consumers(owner_id,consumer,generation,lease_epoch) VALUES('feedback-owner','snapshots-v1',3,5);
INSERT INTO analytics_ingestion_producers(producer,pending_count,checkpoint) VALUES('feedback.fixture',0,17);
INSERT INTO analytics_snapshot_state(owner_id,generation,fact_generation,compacted_before,evidence_cutoff)
 VALUES('feedback-owner',8,3,'2019-01-01','2019-01-01T00:00:00Z');
INSERT INTO analytics_snapshot_daily(owner_id,scope,day,metrics,spend,histogram,cohort,suppressed,generation)
 VALUES('feedback-owner','','2020-01-01','{"requests":3}','{"USD":500}','[0,2,1]',5,false,8);
INSERT INTO analytics_feedback_snapshots(owner_id,scope,window_days,generation,from_day,to_day,body)
 VALUES('feedback-owner','',365,8,'2020-01-01','2021-01-01','{"distinct_users":null,"requests":3}');
INSERT INTO analytics_snapshot_jobs(owner_id,job_id,scope,from_day,to_day,state,lease_epoch,result)
 VALUES('feedback-owner','feedback-range','','2020-01-01','2020-02-01','ready',4,'{"requests":3}');
INSERT INTO eval_campaigns(id,owner_id,created_by,resource_id,baseline_revision_id,budget_id,policy,policy_digest,operation_key)
 VALUES('feedback-campaign','feedback-owner','feedback-owner','feedback-resource','feedback-revision','feedback-budget','{}',repeat('3',64),'feedback-campaign');
INSERT INTO eval_campaign_diagnostics(id,owner_id,campaign_id,operation_key,stage,code,remediation,actor_id,occurrences)
 VALUES('feedback-diagnostic','feedback-owner','feedback-campaign','feedback-launch','admission','native_unavailable','Install verified native runtime','feedback-owner',2);
INSERT INTO eval_campaign_holdout_proposals(id,owner_id,campaign_id,development_experiment_id,operation_key,spec,spec_digest,development_cases,holdout_cases)
 VALUES('feedback-holdout','feedback-owner','feedback-campaign','feedback-experiment','feedback-holdout','{"independent":true}',repeat('4',64),30,10);
INSERT INTO eval_holdout_content_consumption(owner_id,content_digest,experiment_id)
 VALUES('feedback-owner',repeat('5',64),'feedback-experiment');
INSERT INTO plugin_usage_events(id,user_id,session_id,event_type,prompt_preview,created_at)
 VALUES('feedback-raw','feedback-owner','owner-session','UserPromptSubmit','/fixture:example','2020-01-01T00:00:00Z');
INSERT INTO reviewed_production_failures(id,owner_id,invocation_id,reviewer_id,sanitized_evidence,development_case_revision_id)
 VALUES('feedback-reviewed','feedback-owner','feedback-raw','feedback-owner','{"sanitized":true}','feedback-case');

-- API008 recovery states retain fences/checkpoints; issued tokens are never stored.
INSERT INTO managed_api_operations(owner_id,id,kind,request_digest,state,fence,lease_until,input_checkpoint,result,problem)
 VALUES('feedback-owner','feedback-api-pending','source_capture',repeat('6',64),'pending',7,'2020-01-01T00:00:00Z','{"retained":"capture-input"}',NULL,NULL),
 ('feedback-owner','feedback-api-completed','credential_issue',repeat('7',64),'completed',3,'2020-01-01T00:00:00Z',NULL,'{"device_id":"feedback-device","consumer_id":"feedback-consumer","token_retrievable":false,"retry_action":"rotate with a new key"}',NULL),
 ('feedback-owner','feedback-api-failed','source_verification',repeat('8',64),'failed',2,'2020-01-01T00:00:00Z',NULL,NULL,'Retained failure: correct input and use a new operation key');
UPDATE managed_consumer_credentials SET issuance_operation='feedback-api-completed' WHERE device_id='feedback-device';
-- AI024 changes only the failure projection; settled usage/cost/time stay retained.
UPDATE ai_requests SET status='failed',accounting_failed_at='2020-01-02T00:00:00Z',accounting_error='fixture accounting mismatch',error_message='fixture accounting mismatch' WHERE id='feedback-ai-completed';
