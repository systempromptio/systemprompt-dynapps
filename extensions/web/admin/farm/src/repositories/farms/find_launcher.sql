SELECT id AS "id: UserId"
FROM users
WHERE (id = $1 OR lower(email) = lower($1)) AND account_kind <> 'service' AND status = 'active'
