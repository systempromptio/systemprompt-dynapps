public class LeadCountryRoutingEvaluator {
    public static final Set<String> SUPPORTED_OPERATORS = new Set<String>{
        '==', '!=', 'contains', 'startsWith', 'endsWith'
    };

    public class RoutingResult {
        public String targetQueueId;
        public String ruleDevName;
        public String errorMessage;
        public Boolean isMatch;

        public RoutingResult(String targetId, String devName) {
            this.targetQueueId = targetId;
            this.ruleDevName = devName;
            this.isMatch = true;
            this.errorMessage = null;
        }

        public RoutingResult(String error) {
            this.targetQueueId = null;
            this.ruleDevName = null;
            this.isMatch = false;
            this.errorMessage = error;
        }
    }

    @InvocableMethod(label='Evaluate Lead Country Routing' description='Find target queue for a lead based on country')
    public static List<RoutingResult> evaluateRouting(List<Lead> leads) {
        List<RoutingResult> results = new List<RoutingResult>();

        if (leads == null || leads.isEmpty()) {
            return results;
        }

        List<Lead_Country_Routing__mdt> allRules = queryAndSortRules();
        validateRules(allRules);

        for (Lead lead : leads) {
            RoutingResult result = findMatchingRoute(lead, allRules);
            results.add(result);
        }

        return results;
    }

    private static List<Lead_Country_Routing__mdt> queryAndSortRules() {
        List<Lead_Country_Routing__mdt> rules = [
            SELECT DeveloperName, Country__c, Operator__c, Target_Queue_Id__c, Priority__c, Is_Active__c
            FROM Lead_Country_Routing__mdt
            WHERE Is_Active__c = true
            ORDER BY Priority__c ASC, DeveloperName ASC
        ];
        return rules;
    }

    private static void validateRules(List<Lead_Country_Routing__mdt> rules) {
        for (Lead_Country_Routing__mdt rule : rules) {
            if (!SUPPORTED_OPERATORS.contains(rule.Operator__c)) {
                System.debug(LoggingLevel.WARN,
                    'Invalid operator on Lead_Country_Routing__mdt.' + rule.DeveloperName +
                    ': ' + rule.Operator__c);
            }
        }
    }

    private static RoutingResult findMatchingRoute(Lead lead, List<Lead_Country_Routing__mdt> rules) {
        String leadCountry = lead.Country ?? '';

        for (Lead_Country_Routing__mdt rule : rules) {
            if (!SUPPORTED_OPERATORS.contains(rule.Operator__c)) {
                continue;
            }

            if (evaluateCondition(leadCountry, rule.Operator__c, rule.Country__c)) {
                return new RoutingResult(rule.Target_Queue_Id__c, rule.DeveloperName);
            }
        }

        return new RoutingResult('No matching routing rule found for country: ' + leadCountry);
    }

    private static Boolean evaluateCondition(String fieldValue, String operator, String compareValue) {
        if (fieldValue == null || compareValue == null) {
            return false;
        }

        switch on operator {
            when '==' {
                return fieldValue == compareValue;
            }
            when '!=' {
                return fieldValue != compareValue;
            }
            when 'contains' {
                return fieldValue.contains(compareValue);
            }
            when 'startsWith' {
                return fieldValue.startsWith(compareValue);
            }
            when 'endsWith' {
                return fieldValue.endsWith(compareValue);
            }
        }

        return false;
    }
}
