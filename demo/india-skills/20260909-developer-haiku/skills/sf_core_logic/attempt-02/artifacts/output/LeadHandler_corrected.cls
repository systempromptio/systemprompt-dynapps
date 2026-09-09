public with sharing class LeadHandler {
  // Recursion flag explicitly initialized to false
  private static Boolean bypass = false;

  public static void run(List<Lead> records) {
    if (bypass) return;

    bypass = true;
    try {
      // Collect IDs outside DML
      Set<Id> leadIds = new Set<Id>();
      for (Lead item : records) {
        leadIds.add(item.Id);
      }

      // Single SOQL query (not in loop)
      List<Lead> leadsToUpdate = [
        SELECT Id, Country
        FROM Lead
        WHERE Id IN :leadIds
        WITH SECURITY_ENFORCED
      ];

      // Update field values
      for (Lead found : leadsToUpdate) {
        for (Lead item : records) {
          if (found.Id == item.Id) {
            found.Country = item.Country;
            break;
          }
        }
      }

      // Apply FLS; single DML (not in loop)
      List<Lead> accessible = Security.stripInaccessible(
        AccessLevel.SYSTEM_MODE,
        leadsToUpdate
      ).getRecords();

      update accessible;
    } finally {
      bypass = false;
    }
  }
}
