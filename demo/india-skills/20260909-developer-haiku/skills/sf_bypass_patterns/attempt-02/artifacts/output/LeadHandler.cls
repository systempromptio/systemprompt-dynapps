public without sharing class LeadHandler {
    @TestVisible private static Boolean bypassForTest = false;

    public static Boolean isBypassed() {
        if (Test.isRunningTest() && bypassForTest) {
            return true;
        }
        return FeatureManagement.checkPermission('Lead_Update_Country_Bypass');
    }

    public static void run(List<Lead> records) {
        if (isBypassed()) {
            return;
        }
        Map<Id, Lead> currentLeads = new Map<Id, Lead>(
            [SELECT Id, Country FROM Lead WHERE Id IN :records]
        );
        List<Lead> toUpdate = new List<Lead>();
        for (Lead item : records) {
            Lead found = currentLeads.get(item.Id);
            if (found != null && found.Country != item.Country) {
                found.Country = item.Country;
                toUpdate.add(found);
            }
        }
        if (!toUpdate.isEmpty()) {
            update toUpdate;
        }
    }
}
