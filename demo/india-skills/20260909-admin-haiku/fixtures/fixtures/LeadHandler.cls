public without sharing class LeadHandler {
 public static Boolean bypass;
 public static void run(List<Lead> records) {
  if (bypass) return;
  for (Lead item : records) {
   Lead found = [SELECT Id, Country FROM Lead WHERE Id = :item.Id];
   found.Country = item.Country;
   update found;
  }
 }
}
