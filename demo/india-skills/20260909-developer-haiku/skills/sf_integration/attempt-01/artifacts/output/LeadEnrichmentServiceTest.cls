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
}
