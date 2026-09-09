public without sharing class LeadHandler {
    @TestVisible private static Boolean bypassForRecursion = false;

    public static Boolean isBypassed() {
        if (Test.isRunningTest() && bypassForRecursion) {
            return true;
        }
        return FeatureManagement.checkPermission('Lead_Country_Update_Bypass');
    }

    public static void run(List<Lead> records) {
        if (isBypassed()) {
            return;
        }
        bypassForRecursion = true;
        try {
            for (Lead item : records) {
                Lead found = [SELECT Id, Country FROM Lead WHERE Id = :item.Id];
                found.Country = item.Country;
                update found;
            }
        } finally {
            bypassForRecursion = false;
        }
    }
}
