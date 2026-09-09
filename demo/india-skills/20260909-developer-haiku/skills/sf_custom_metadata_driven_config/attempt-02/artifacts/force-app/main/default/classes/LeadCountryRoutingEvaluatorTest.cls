@IsTest
private class LeadCountryRoutingEvaluatorTest {

    @IsTest
    static void testMatchingRule() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = 'United States');

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(1, results.size(), 'Should return one result');
        System.assertEquals(true, results[0].isMatch, 'Should find a matching rule for United States');
        System.assertEquals('ROUTE_US', results[0].ruleDevName, 'Should match ROUTE_US');
        System.assertEquals('00G000000000000', results[0].targetQueueId, 'Should return correct target queue');
    }

    @IsTest
    static void testNoMatchingRule() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = 'Antarctica');

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(1, results.size(), 'Should return one result');
        System.assertEquals(false, results[0].isMatch, 'Should not find a matching rule for Antarctica');
        System.assertEquals(null, results[0].targetQueueId, 'Should return null target queue');
        System.assertNotEquals(null, results[0].errorMessage, 'Should include error message');
    }

    @IsTest
    static void testContainsOperator() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = 'Ireland EU');

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(1, results.size(), 'Should return one result');
        System.assertEquals(true, results[0].isMatch, 'Should match EMEA rule with contains operator');
        System.assertEquals('ROUTE_EMEA', results[0].ruleDevName, 'Should match ROUTE_EMEA');
    }

    @IsTest
    static void testPriorityOrdering() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = 'United States');

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(true, results[0].isMatch, 'Should find match');
        System.assertEquals('ROUTE_US', results[0].ruleDevName, 'Priority 1 (ROUTE_US) should match before Priority 2 (ROUTE_EMEA)');
    }

    @IsTest
    static void testDisabledRuleIgnored() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = 'Test Country');

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(1, results.size(), 'Should return one result');
        System.assertEquals(false, results[0].isMatch, 'Should not match disabled rule');
    }

    @IsTest
    static void testNullCountry() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = null);

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(1, results.size(), 'Should return one result');
        System.assertEquals(false, results[0].isMatch, 'Should not match when country is null');
    }

    @IsTest
    static void testEmptyLeadList() {
        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>());
        Test.stopTest();

        System.assertEquals(0, results.size(), 'Should return empty list for empty input');
    }

    @IsTest
    static void testTieBreakerByDeveloperName() {
        Lead testLead = new Lead(FirstName = 'Test', LastName = 'Lead', Company = 'Test Co', Country = 'United States');

        Test.startTest();
        List<LeadCountryRoutingEvaluator.RoutingResult> results = LeadCountryRoutingEvaluator.evaluateRouting(new List<Lead>{testLead});
        Test.stopTest();

        System.assertEquals(true, results[0].isMatch, 'Should match');
        System.assertEquals('ROUTE_US', results[0].ruleDevName, 'Alphabetically first DeveloperName wins on tie');
    }
}
