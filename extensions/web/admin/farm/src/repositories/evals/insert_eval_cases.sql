INSERT INTO agent_farm_eval_cases (suite_id, id, prompt, fixture, timeout_secs, assertions,
                                   judge_rubric, judge_threshold)
SELECT $1, c.id, c.prompt, c.fixture, c.timeout_secs, c.assertions, c.judge_rubric,
       c.judge_threshold
FROM UNNEST($2::text[], $3::text[], $4::text[], $5::int4[], $6::jsonb[], $7::text[],
            $8::float8[])
    AS c(id, prompt, fixture, timeout_secs, assertions, judge_rubric, judge_threshold)
