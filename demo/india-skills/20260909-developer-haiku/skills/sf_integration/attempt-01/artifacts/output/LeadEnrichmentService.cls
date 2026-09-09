public class LeadEnrichmentService {
    private static final String NAMED_CREDENTIAL = 'callout:LeadEnrichmentAPI';
    private static final String ENRICHMENT_ENDPOINT = '/leads/enrich';
    private static final Integer TIMEOUT_MS = 10000;

    public static void enrichLeadsAsync(List<Lead> leads) {
        List<Lead> leadsToEnrich = new List<Lead>();
        for (Lead l : leads) {
            if (String.isNotBlank(l.Email)) {
                leadsToEnrich.add(l);
            }
        }
        if (!leadsToEnrich.isEmpty()) {
            System.enqueueJob(new LeadEnrichmentQueueable(leadsToEnrich));
        }
    }

    public static Map<String, Object> callEnrichmentAPI(String leadId, String email) {
        HttpRequest req = new HttpRequest();
        req.setEndpoint(NAMED_CREDENTIAL + ENRICHMENT_ENDPOINT);
        req.setMethod('POST');
        req.setHeader('Content-Type', 'application/json');
        req.setTimeout(TIMEOUT_MS);

        Map<String, Object> payload = new Map<String, Object>{
            'leadId' => leadId,
            'email' => email,
            'timestamp' => System.now().getTime()
        };
        req.setBody(JSON.serialize(payload));

        try {
            Http http = new Http();
            HttpResponse res = http.send(req);

            if (res.getStatusCode() == 200) {
                return (Map<String, Object>) JSON.deserializeUntyped(res.getBody());
            } else {
                logFailure(leadId, 'API returned status ' + res.getStatusCode());
                return null;
            }
        } catch (Exception e) {
            logFailure(leadId, e.getMessage());
            return null;
        }
    }

    private static void logFailure(String leadId, String reason) {
        System.debug(LoggingLevel.ERROR, 'Lead enrichment failed for ' + leadId + ': ' + reason);
    }

    public static void updateLeadFromEnrichment(Lead lead, Map<String, Object> enrichmentData) {
        if (enrichmentData != null) {
            if (enrichmentData.containsKey('company')) {
                lead.Company = (String) enrichmentData.get('company');
            }
            if (enrichmentData.containsKey('country')) {
                lead.Country = (String) enrichmentData.get('country');
            }
            if (enrichmentData.containsKey('phone')) {
                lead.Phone = (String) enrichmentData.get('phone');
            }
        }
    }
}
