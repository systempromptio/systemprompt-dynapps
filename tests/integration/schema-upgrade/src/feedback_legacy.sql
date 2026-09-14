-- Deliberately valid against the recorded release, before feedback migrations.
INSERT INTO users(id,name,email,roles) VALUES
 ('feedback-owner','Owner','feedback-owner@example.test',ARRAY['admin']),
 ('feedback-consumer','Consumer','feedback-consumer@example.test',ARRAY['user']);
INSERT INTO user_device_certs(id,user_id,fingerprint,label)
 VALUES('feedback-device','feedback-consumer','feedback-fingerprint','fixture');
INSERT INTO managed_sources(id,owner_id,name,kind,specification)
 VALUES('feedback-source','feedback-owner','Private source','git','{"url":"https://example.test/private.git"}');
INSERT INTO managed_resources(id,owner_id,source_id,upstream_key,kind,resource_key)
 VALUES('feedback-resource','feedback-owner','feedback-source','skills/example','skill','example');
INSERT INTO managed_source_snapshots(id,owner_id,source_id,digest,provenance)
 VALUES('feedback-source-snapshot','feedback-owner','feedback-source',repeat('a',64),'{"commit":"retained"}');
INSERT INTO managed_revisions(id,owner_id,resource_id,source_id,snapshot_id,digest,manifest,rationale)
 VALUES('feedback-revision','feedback-owner','feedback-resource','feedback-source','feedback-source-snapshot',repeat('b',64),'{"bytes":"immutable","mode":493}','Retained candidate');
INSERT INTO managed_publication_reviews(id,owner_id,resource_id,revision_id,action,bundle_digest,comparison_evidence,limitations,reviewer_id,expected_generation,request_digest)
 VALUES('feedback-review','feedback-owner','feedback-resource','feedback-revision','initial_adoption',repeat('c',64),'{"human":true}','Native acceptance unavailable','feedback-owner',0,repeat('d',64));
INSERT INTO managed_publications(id,owner_id,resource_id,review_id,generation,action,revision_id,bundle_digest,operation_key,request_digest)
 VALUES('feedback-publication','feedback-owner','feedback-resource','feedback-review',1,'initial_adoption','feedback-revision',repeat('c',64),'feedback-publish',repeat('d',64));
INSERT INTO managed_installation_receipts(id,owner_id,installation_id,publication_id,resource_id,generation,bundle_digest,installed_manifest,client_evidence)
 VALUES('feedback-legacy-receipt','feedback-owner','feedback-installation','feedback-publication','feedback-resource',1,repeat('c',64),'{"path":"SKILL.md"}','{"historical":true}');
INSERT INTO eval_budget_accounts(id,owner_id,cap,reserved,settled,operation_key)
 VALUES('feedback-budget','feedback-owner',100000,1000,500,'feedback-budget');
INSERT INTO eval_resource_revisions(id,owner_id,resource_kind,resource_key,digest,content)
 VALUES('feedback-case','feedback-owner','case','fixture',repeat('e',64),'{"independent":true}');
INSERT INTO eval_experiments(id,owner_id,spec,spec_digest,budget_id,idempotency_key)
 VALUES('feedback-experiment','feedback-owner','{"client":"opencode","version":"pinned-fixture"}',repeat('f',64),'feedback-budget','feedback-experiment');
INSERT INTO eval_executions(id,experiment_id,variant_index,case_revision_id,repetition,status,fencing_token,result)
 VALUES('feedback-execution','feedback-experiment',0,'feedback-case',0,'completed',7,'{"native_isolation":"unavailable","enabled":false}');
INSERT INTO eval_execution_evidence(execution_id,fencing_token,digest,manifest)
 VALUES('feedback-execution',7,repeat('f',64),'{"executable_digest":"retained","client":"opencode","cleanup_verified":true}');
INSERT INTO eval_execution_cleanup(execution_id,status,attempts) VALUES('feedback-execution','verified',2);
INSERT INTO eval_execution_measurements(execution_id,attempted_cost_microdollars,accounting_status,verified_success)
 VALUES('feedback-execution',500,'partial',false);

-- These columns are present in the recorded release (no AI024 fields here).
INSERT INTO ai_requests(id,request_id,user_id,context_id,provider,model,actor_kind,actor_id,status,input_tokens,output_tokens,tokens_used,cost_microdollars,completed_at,created_at,updated_at)
 VALUES('feedback-ai-completed','feedback-provider-request','feedback-owner','feedback-context','fixture','fixture-model','user','feedback-owner','completed',11,7,18,500,'2020-01-01T00:00:00Z','2020-01-01T00:00:00Z','2020-01-01T00:00:00Z');
