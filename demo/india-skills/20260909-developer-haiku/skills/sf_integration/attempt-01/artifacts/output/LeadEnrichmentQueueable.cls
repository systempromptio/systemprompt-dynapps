public class LeadEnrichmentQueueable implements Queueable, Database.AllowsCallouts {
    private List<Lead> leads;
    private Integer retryCount;
    private static final Integer MAX_RETRIES = 3;
    private static final Integer BATCH_SIZE = 100;

    public LeadEnrichmentQueueable(List<Lead> leads) {
        this.leads = leads;
        this.retryCount = 0;
    }

    public LeadEnrichmentQueueable(List<Lead> leads, Integer retryCount) {
        this.leads = leads;
        this.retryCount = retryCount;
    }

    public void execute(QueueableContext context) {
        List<Lead> leadsToUpdate = new List<Lead>();
        List<Lead> failedLeads = new List<Lead>();

        for (Lead lead : leads) {
            String idempotencyKey = generateIdempotencyKey(lead.Id, lead.Email);

            Map<String, Object> enrichmentResult = LeadEnrichmentService.callEnrichmentAPI(lead.Id, lead.Email);

            if (enrichmentResult != null) {
                LeadEnrichmentService.updateLeadFromEnrichment(lead, enrichmentResult);
                leadsToUpdate.add(lead);
                recordSuccess(lead.Id, idempotencyKey);
            } else {
                failedLeads.add(lead);
            }
        }

        if (!leadsToUpdate.isEmpty()) {
            update leadsToUpdate;
        }

        if (!failedLeads.isEmpty() && retryCount < MAX_RETRIES) {
            System.enqueueJob(new LeadEnrichmentQueueable(failedLeads, retryCount + 1));
        } else if (!failedLeads.isEmpty()) {
            for (Lead l : failedLeads) {
                System.debug(LoggingLevel.ERROR, 'Lead ' + l.Id + ' exhausted retries');
            }
        }
    }

    private String generateIdempotencyKey(String leadId, String email) {
        String combined = leadId + ':' + email;
        return EncodingUtil.convertToHex(
            Crypto.generateDigest('SHA1', Blob.valueOf(combined))
        );
    }

    private void recordSuccess(String leadId, String idempotencyKey) {
        System.debug(LoggingLevel.INFO, 'Enriched lead ' + leadId + ' with key ' + idempotencyKey);
    }
}
