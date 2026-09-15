SELECT id AS resource_id,resource_key,kind FROM managed_resources WHERE owner_id=$1 AND id=$2
