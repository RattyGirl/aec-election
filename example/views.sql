--VIEWS
--
--
--
--

CREATE VIEW pollingplacetotal AS (
 SELECT event_id,
    contest_id,
    polling_place_id,
    sum(votes) AS sum
   FROM primaryvote
  GROUP BY event_id, contest_id, polling_place_id);


CREATE VIEW results AS (
 SELECT p.timestamp,
    p.event_id,
    p.contest_id,
    p.polling_place_id,
    c.name,
    c.affiliation,
    p.votes
   FROM primaryvote p
     LEFT JOIN candidate c ON p.candidate_id::text = c.id::text AND p.event_id::text = c.event_id::text AND p.contest_id::text = c.contest_id::text);


CREATE VIEW resultstotal AS (
 SELECT r.event_id,
    r.contest_id,
    r.polling_place_id,
    r.name,
    r.affiliation,
    r.votes,
    t.sum,
    r.timestamp
   FROM results r
     LEFT JOIN pollingplacetotal t ON r.event_id::text = t.event_id::text AND r.contest_id::text = t.contest_id::text AND r.polling_place_id::text = t.polling_place_id::text);

--TWO CANDIDATE PREFERRED
CREATE VIEW two_pollingplacetotal AS (
 SELECT event_id,
    contest_id,
    polling_place_id,
    sum(votes) AS sum
   FROM twocandidatepreferredvote
  GROUP BY event_id, contest_id, polling_place_id);


CREATE VIEW two_results AS (
 SELECT p.timestamp,
    p.event_id,
    p.contest_id,
    p.polling_place_id,
    c.name,
    c.affiliation,
    p.votes
   FROM twocandidatepreferredvote p
     LEFT JOIN candidate c ON p.candidate_id::text = c.id::text AND p.event_id::text = c.event_id::text AND p.contest_id::text = c.contest_id::text);


CREATE VIEW two_resultstotal AS (
 SELECT r.event_id,
    r.contest_id,
    r.polling_place_id,
    r.name,
    r.affiliation,
    r.votes,
    t.sum,
    r.timestamp
   FROM two_results r
     LEFT JOIN two_pollingplacetotal t ON r.event_id::text = t.event_id::text AND r.contest_id::text = t.contest_id::text AND r.polling_place_id::text = t.polling_place_id::text);
