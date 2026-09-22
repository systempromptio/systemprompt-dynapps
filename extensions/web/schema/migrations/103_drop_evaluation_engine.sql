-- The evaluation engine's storage, dropped again.
--
-- Migration 085 already dropped these four views and three of these tables,
-- and 074 dropped session_analyses; all are stamped applied. They are
-- nonetheless live and empty, so a drop that ran once did not reach every
-- database -- restoring a dump older than the drop puts the whole set back,
-- and nothing since notices. The declarative schema has not declared any of
-- them since the engine was removed, which is why boot reports the tables as
-- owned by no extension. The views were never reported: the residue audit
-- reads base tables only.
--
-- Views before tables, in 085's order, because evaluation_session_scorecard
-- reads evaluation_session_scores and every view reads evaluation_runs.
-- Dropping the tables first would need CASCADE, which would take whatever
-- else had come to depend on them without naming it.
--
-- Every statement is a DROP ... IF EXISTS, so this is a retirement migration:
-- executed as well as stamped on a fresh extension, and it converges whether
-- the objects are present or not.
DROP VIEW IF EXISTS evaluation_version_scores;
DROP VIEW IF EXISTS evaluation_session_skills;
DROP VIEW IF EXISTS evaluation_session_scorecard;
DROP VIEW IF EXISTS evaluation_session_scores;
DROP TABLE IF EXISTS evaluation_reports;
DROP TABLE IF EXISTS evaluation_runs;
DROP TABLE IF EXISTS evaluation_queue;
DROP TABLE IF EXISTS session_analyses;
