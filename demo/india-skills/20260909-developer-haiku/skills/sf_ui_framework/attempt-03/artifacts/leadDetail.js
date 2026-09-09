import { LightningElement, api, wire } from 'lwc';
import { getRecord, getFieldValue } from 'lightning/uiRecordApi';

const FIELDS = ['Lead.Name', 'Lead.Status'];

export default class LeadDetail extends LightningElement {
  @api recordId;

  isLoading = false;
  error = null;
  leadRecord = null;

  @wire(getRecord, {
    recordId: '$recordId',
    fields: FIELDS
  })
  wiredLead({ error, data }) {
    this.isLoading = false;

    if (error) {
      this.error = 'Failed to load lead. Please try again.';
      this.leadRecord = null;
    } else if (data) {
      this.leadRecord = data;
      this.error = null;
    }
  }

  get hasRecord() {
    return this.leadRecord != null;
  }

  get leadName() {
    return getFieldValue(this.leadRecord, 'Lead.Name') || '—';
  }

  get leadStatus() {
    return getFieldValue(this.leadRecord, 'Lead.Status') || '—';
  }
}
