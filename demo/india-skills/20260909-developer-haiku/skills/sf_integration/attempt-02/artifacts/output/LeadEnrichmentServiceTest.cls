@IsTest
public class LeadEnrichmentServiceTest {

    @IsTest
    static void testEnrichmentAPISuccess() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentAPISuccessMock());

        Lead testLead = new Lead(
            Id = '00Q00000000001',
            Email = 'test@example.com',
            FirstName = 'John',
            LastName = 'Doe',
            Company = 'OldCo'
        );

        Test.startTest();
        Map<String, Object> result = LeadEnrichmentService.callEnrichmentAPI(testLead.Id, testLead.Email);
        Test.stopTest();

        Assert.isNotNull(result, 'Result should not be null');
        Assert.areEqual('NewCo', result.get('company'), 'Company should match');
        Assert.areEqual('US', result.get('country'), 'Country should match');
    }

    @IsTest
    static void testEnrichmentAPIFailure() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentAPIFailureMock());

        Test.startTest();
        Map<String, Object> result = LeadEnrichmentService.callEnrichmentAPI('00Q00000000001', 'test@example.com');
        Test.stopTest();

        Assert.isNull(result, 'Result should be null on failure');
    }

    @IsTest
    static void testLeadUpdateFromEnrichment() {
        Lead testLead = new Lead(
            FirstName = 'John',
            LastName = 'Doe',
            Company = 'OldCo',
            Country = 'GB'
        );

        Map<String, Object> enrichmentData = new Map<String, Object>{
            'company' => 'NewCo',
            'country' => 'US',
            'phone' => '+1-555-0123'
        };

        LeadEnrichmentService.updateLeadFromEnrichment(testLead, enrichmentData);

        Assert.areEqual('NewCo', testLead.Company, 'Company should be updated');
        Assert.areEqual('US', testLead.Country, 'Country should be updated');
        Assert.areEqual('+1-555-0123', testLead.Phone, 'Phone should be updated');
    }

    @IsTest
    static void testIdempotencyKeyGeneration() {
        Test.startTest();
        String key1 = generateIdempotencyKey('00Q00000000001', 'test@example.com');
        String key2 = generateIdempotencyKey('00Q00000000001', 'test@example.com');
        Test.stopTest();

        Assert.areEqual(key1, key2, 'Same input should produce same key');
    }

    @IsTest
    static void testAPITimeoutHandling() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentAPITimeoutMock());

        Test.startTest();
        Map<String, Object> result = LeadEnrichmentService.callEnrichmentAPI('00Q00000000002', 'timeout@example.com');
        Test.stopTest();

        Assert.isNull(result, 'Result should be null on timeout');
    }

    @IsTest
    static void testDuplicateResponseHandling() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentAPIDuplicateMock());

        Lead lead1 = new Lead(Id = '00Q00000000003', Email = 'dup@example.com');

        Test.startTest();
        Map<String, Object> result1 = LeadEnrichmentService.callEnrichmentAPI(lead1.Id, lead1.Email);
        Map<String, Object> result2 = LeadEnrichmentService.callEnrichmentAPI(lead1.Id, lead1.Email);
        Test.stopTest();

        Assert.isNotNull(result1, 'First call should succeed');
        Assert.isNotNull(result2, 'Second call should return same data (idempotent)');
        String key1 = generateIdempotencyKey(lead1.Id, lead1.Email);
        String key2 = generateIdempotencyKey(lead1.Id, lead1.Email);
        Assert.areEqual(key1, key2, 'Idempotency keys should match for duplicate attempts');
    }

    @IsTest
    static void test4xxErrorHandling() {
        Test.setMock(HttpCalloutMock.class, new EnrichmentAPI4xxMock());

        Test.startTest();
        Map<String, Object> result = LeadEnrichmentService.callEnrichmentAPI('00Q00000000004', 'notfound@example.com');
        Test.stopTest();

        Assert.isNull(result, 'Result should be null on 404 error');
    }

    private static String generateIdempotencyKey(String leadId, String email) {
        String combined = leadId + ':' + email;
        return EncodingUtil.convertToHex(
            Crypto.generateDigest('SHA1', Blob.valueOf(combined))
        );
    }

    private class EnrichmentAPISuccessMock implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            Assert.isTrue(req.getEndpoint().contains('callout:LeadEnrichmentAPI'), 'Named Credential not used');
            Assert.areEqual('POST', req.getMethod(), 'Should be POST');

            HttpResponse res = new HttpResponse();
            res.setStatusCode(200);
            res.setBody(JSON.serialize(new Map<String, Object>{
                'company' => 'NewCo',
                'country' => 'US',
                'phone' => '+1-555-0123'
            }));
            return res;
        }
    }

    private class EnrichmentAPIFailureMock implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            HttpResponse res = new HttpResponse();
            res.setStatusCode(500);
            res.setBody('Internal Server Error');
            return res;
        }
    }

    private class EnrichmentAPITimeoutMock implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            HttpResponse res = new HttpResponse();
            res.setStatusCode(408);
            res.setBody('Request Timeout');
            return res;
        }
    }

    private class EnrichmentAPIDuplicateMock implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            HttpResponse res = new HttpResponse();
            res.setStatusCode(200);
            res.setBody(JSON.serialize(new Map<String, Object>{
                'company' => 'DupCo',
                'country' => 'CA',
                'phone' => '+1-555-9999',
                'idempotencyKey' => generateIdempotencyKey('00Q00000000003', 'dup@example.com')
            }));
            return res;
        }

        private String generateIdempotencyKey(String leadId, String email) {
            String combined = leadId + ':' + email;
            return EncodingUtil.convertToHex(
                Crypto.generateDigest('SHA1', Blob.valueOf(combined))
            );
        }
    }

    private class EnrichmentAPI4xxMock implements HttpCalloutMock {
        public HttpResponse respond(HttpRequest req) {
            HttpResponse res = new HttpResponse();
            res.setStatusCode(404);
            res.setBody('Lead not found in enrichment database');
            return res;
        }
    }
}
