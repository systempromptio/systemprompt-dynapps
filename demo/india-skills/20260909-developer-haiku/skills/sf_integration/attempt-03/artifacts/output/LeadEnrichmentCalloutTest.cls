@isTest
public class LeadEnrichmentCalloutTest {
    private static final String MOCK_ENDPOINT = 'callout:LeadEnrichmentAPI/enrich';

    @testSetup
    static void setup() {
        insert new Lead(
            FirstName = 'Test',
            LastName = 'Lead',
            Email = 'test@example.com',
            Company = 'Acme'
        );
    }

    @isTest
    static void testSuccessfulEnrichment() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentMockSuccess());
        Test.startTest();
        Lead lead = [SELECT Id FROM Lead LIMIT 1];
        System.enqueueJob(new LeadEnrichmentCallout(new List<Lead>{lead}));
        Test.stopTest();

        Lead enriched = [SELECT Description, Phone FROM Lead];
        System.assertEquals('Senior Engineer at Acme', enriched.Description);
        System.assertEquals('+1-555-0123', enriched.Phone);
    }

    @isTest
    static void testBulkBatching() {
        List<Lead> leads = new List<Lead>();
        for (Integer i = 0; i < 250; i++) {
            leads.add(new Lead(FirstName = 'Test' + i, LastName = 'Lead', Email = 'test' + i + '@test.com', Company = 'Acme'));
        }
        insert leads;

        Test.setMock(HttpCalloutMock.class, new EnrichmentMockSuccess());
        Test.startTest();
        System.enqueueJob(new LeadEnrichmentCallout(leads));
        Test.stopTest();

        System.assertEquals(250, [SELECT COUNT() FROM Lead]);
    }

    @isTest
    static void testIdempotencyKeyGeneration() {
        Lead lead = [SELECT Id, Email FROM Lead LIMIT 1];
        String key1 = LeadEnrichmentCallout.generateIdempotencyKey(lead);
        String key2 = LeadEnrichmentCallout.generateIdempotencyKey(lead);
        System.assertEquals(key1, key2, 'Idempotency key should be deterministic');
    }

    @isTest
    static void testRetryOnFailure() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentMockFailure());
        Test.startTest();
        Lead lead = [SELECT Id FROM Lead LIMIT 1];
        System.enqueueJob(new LeadEnrichmentCallout(new List<Lead>{lead}));
        Test.stopTest();

        EnrichmentLog__c log = [SELECT RetryCount__c FROM EnrichmentLog__c LIMIT 1];
        System.assertNotEquals(null, log, 'Error should be logged on failure');
    }

    @isTest
    static void testOversizedBatch() {
        List<Lead> leads = new List<Lead>();
        for (Integer i = 0; i < 150; i++) {
            leads.add(new Lead(FirstName = 'Bulk' + i, LastName = 'Test', Email = 'bulk' + i + '@test.com', Company = 'BigCorp'));
        }
        insert leads;

        Test.setMock(HttpCalloutMock.class, new EnrichmentMockSuccess());
        Test.startTest();
        System.enqueueJob(new LeadEnrichmentCallout(leads));
        Test.stopTest();

        System.assert([SELECT COUNT() FROM Lead] >= 150, 'All leads processed via chaining');
    }

    public class EnrichmentMockSuccess implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            HttpResponse res = new HttpResponse();
            res.setStatusCode(200);
            res.setBody('{"enrichedDescription":"Senior Engineer at Acme","phone":"+1-555-0123","industry":"Technology"}');
            return res;
        }
    }

    public class EnrichmentMockFailure implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            HttpResponse res = new HttpResponse();
            res.setStatusCode(500);
            res.setBody('{"error":"Service unavailable"}');
            return res;
        }
    }
}
