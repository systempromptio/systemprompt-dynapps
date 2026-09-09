import { LightningElement, api, wire } from 'lwc';
import { getRecord, getFieldValue } from 'lightning/uiRecordApi';

const FIELDS = [
  'Lead.Name',
  'Lead.Status',
];

export default class LeadDisplay extends LightningElement {
  @api recordId;

  leadRecord;
  isLoading = false;
  hasError = false;
  errorMessage = '';

  @wire(getRecord, { recordId: '$recordId', fields: FIELDS })
  wiredLead({ error, data }) {
    this.isLoading = false;
    if (data) {
      this.leadRecord = data;
      this.hasError = false;
      this.errorMessage = '';
    } else if (error) {
      this.hasError = true;
      this.errorMessage = 'Failed to load lead record';
      this.leadRecord = undefined;
    }
  }

  handleRefresh() {
    this.isLoading = true;
    return this.refresh();
  }

  refresh() {
    return this.template.querySelector('lightning-record-form')?.refresh() || Promise.resolve();
  }

  get hasData() {
    return this.leadRecord && !this.isLoading && !this.hasError;
  }
}
