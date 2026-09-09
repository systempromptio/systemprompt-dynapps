public with sharing class LeadAssignmentInvocable {

    public class Request {
        @InvocableVariable(required=true)
        public Id leadId;
    }

    @InvocableMethod(label='Run Lead Assignment Rules' description='Triggers default assignment rules for the provided Lead IDs')
    public static void runAssignmentRules(List<Request> requests) {
        Set<Id> leadIds = new Set<Id>();
        for (Request r : requests) {
            if (r.leadId != null) { leadIds.add(r.leadId); }
        }
        if (leadIds.isEmpty()) { return; }

        List<Lead> leads = [SELECT Id FROM Lead WHERE Id IN :leadIds];
        if (leads.isEmpty()) { return; }

        Database.DMLOptions opts = new Database.DMLOptions();
        opts.assignmentRuleHeader.useDefaultRule = true;
        Database.update(leads, opts);
    }
}
