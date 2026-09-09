public class LeadEnrichmentCallout implements Queueable, Database.AllowsCallouts {
    private static final String NAMED_CREDENTIAL = 'callout:LeadEnrichmentAPI';
    private static final Integer BATCH_SIZE = 100;
    private static final Integer MAX_RETRIES = 3;

    private List<Lead> leadsToEnrich;
    private Integer retryCount = 0;

    public LeadEnrichmentCallout(List<Lead> leads) {
        this.leadsToEnrich = leads;
    }

    public LeadEnrichmentCallout(List<Lead> leads, Integer retries) {
        this.leadsToEnrich = leads;
        this.retryCount = retries;
    }

    public void execute(QueueableContext context) {
        if (leadsToEnrich == null || leadsToEnrich.isEmpty()) {
            return;
        }

        List<Lead> batch = leadsToEnrich.size() > BATCH_SIZE
            ? leadsToEnrich.subList(0, BATCH_SIZE)
            : leadsToEnrich;

        try {
            enrichLeads(batch);
            Database.update(batch, false);

            if (leadsToEnrich.size() > BATCH_SIZE) {
                List<Lead> remaining = leadsToEnrich.subList(BATCH_SIZE, leadsToEnrich.size());
                System.enqueueJob(new LeadEnrichmentCallout(remaining));
            }
        } catch (Exception e) {
            if (retryCount < MAX_RETRIES) {
                System.enqueueJob(new LeadEnrichmentCallout(batch, retryCount + 1));
            }
            logError(e, batch);
        }
    }

    private void enrichLeads(List<Lead> batch) {
        for (Lead lead : batch) {
            String idempotencyKey = generateIdempotencyKey(lead);
            HttpRequest req = buildRequest(lead, idempotencyKey);

            HttpResponse res = new Http().send(req);
            if (res.getStatusCode() == 200) {
                Map<String, Object> response = (Map<String, Object>) JSON.deserializeUntyped(res.getBody());
                mapEnrichmentData(lead, response);
            } else {
                throw new CalloutException('Enrichment API returned: ' + res.getStatusCode());
            }
        }
    }

    private HttpRequest buildRequest(Lead lead, String idempotencyKey) {
        HttpRequest req = new HttpRequest();
        req.setEndpoint(NAMED_CREDENTIAL + '/enrich');
        req.setMethod('POST');
        req.setHeader('Content-Type', 'application/json');
        req.setHeader('Idempotency-Key', idempotencyKey);

        Map<String, Object> body = new Map<String, Object>{
            'email' => lead.Email,
            'firstName' => lead.FirstName,
            'lastName' => lead.LastName,
            'company' => lead.Company
        };
        req.setBody(JSON.serialize(body));
        req.setTimeout(10000);
        return req;
    }

    private String generateIdempotencyKey(Lead lead) {
        return EncodingUtil.convertToHex(Crypto.generateDigest('SHA256', Blob.valueOf(lead.Id + lead.Email)));
    }

    private void mapEnrichmentData(Lead lead, Map<String, Object> data) {
        lead.Description = (String) data.get('enrichedDescription');
        lead.Phone = (String) data.get('phone');
        lead.Industry = (String) data.get('industry');
    }

    private void logError(Exception e, List<Lead> failedLeads) {
        try {
            insert new EnrichmentLog__c(
                Message__c = e.getMessage(),
                LeadIds__c = String.join(new List<Id>(new Map<Id, Lead>(failedLeads).keySet()), ','),
                RetryCount__c = retryCount
            );
        } catch (Exception logEx) {
            System.debug('Failed to log enrichment error: ' + logEx.getMessage());
        }
    }
}
