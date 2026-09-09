import { createElement } from 'lwc';
import LeadDetail from 'c/leadDetail';
import { getRecord } from 'lightning/uiRecordApi';

const mockLead = {
  id: '00Q5g00000H1zH1EAJ',
  fields: {
    Name: { value: 'John Smith' },
    Status: { value: 'Open' }
  }
};

jest.mock('lightning/uiRecordApi', () => ({
  getRecord: jest.fn(),
  getFieldValue: (record, field) => {
    if (!record) return null;
    const fieldPath = field.split('.');
    return record.fields[fieldPath[1]]?.value || null;
  }
}));

describe('c-lead-detail', () => {
  afterEach(() => {
    while (document.body.firstChild) {
      document.body.removeChild(document.body.firstChild);
    }
    jest.clearAllMocks();
  });

  describe('loading state', () => {
    it('displays loading spinner when data is being fetched', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockResolvedValue(mockLead);

      return Promise.resolve().then(() => {
        const spinner = element.shadowRoot.querySelector('lightning-spinner');
        expect(spinner).toBeTruthy();
      });
    });
  });

  describe('load success', () => {
    it('renders lead name and status when record loads', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockResolvedValue(mockLead);

      return Promise.resolve().then(() => {
        const nameField = element.shadowRoot.querySelector('#lead-name');
        const statusField = element.shadowRoot.querySelector('#lead-status');
        expect(nameField.textContent).toBe('John Smith');
        expect(statusField.textContent).toBe('Open');
      });
    });

    it('displays fallback value if field is missing', () => {
      const incompleteRecord = {
        id: mockLead.id,
        fields: { Name: { value: 'Jane Doe' } }
      };

      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockResolvedValue(incompleteRecord);

      return Promise.resolve().then(() => {
        const statusField = element.shadowRoot.querySelector('#lead-status');
        expect(statusField.textContent).toBe('—');
      });
    });
  });

  describe('error state', () => {
    it('displays error message when record fetch fails', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockRejectedValue(new Error('Network error'));

      return Promise.resolve().then(() => {
        const errorElement = element.shadowRoot.querySelector('lightning-illustration');
        expect(errorElement).toBeTruthy();
        expect(errorElement.messageTitle).toBe('Unable to Load Lead');
      });
    });

    it('clears error when record loads after failure', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockRejectedValue(new Error('Network error'));

      return Promise.resolve()
        .then(() => {
          let errorElement = element.shadowRoot.querySelector('lightning-illustration');
          expect(errorElement).toBeTruthy();

          getRecord.mockResolvedValue(mockLead);
          element.recordId = mockLead.id;
        })
        .then(() => {
          let errorElement = element.shadowRoot.querySelector('lightning-illustration');
          expect(errorElement).toBeFalsy();
        });
    });
  });

  describe('keyboard interaction', () => {
    it('allows keyboard focus on lead name field', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockResolvedValue(mockLead);

      return Promise.resolve().then(() => {
        const nameField = element.shadowRoot.querySelector('#lead-name');
        expect(nameField.getAttribute('role')).toBe('textbox');
        expect(nameField.getAttribute('aria-readonly')).toBe('true');
      });
    });

    it('supports tab navigation between fields', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockResolvedValue(mockLead);

      return Promise.resolve().then(() => {
        const fields = element.shadowRoot.querySelectorAll('[id^="lead-"]');
        expect(fields.length).toBe(2);
        fields.forEach(field => {
          expect(field.getAttribute('role')).toBeTruthy();
        });
      });
    });

    it('provides accessible labels for read-only fields', () => {
      const element = createElement('c-lead-detail', { is: LeadDetail });
      element.recordId = mockLead.id;
      document.body.appendChild(element);

      getRecord.mockResolvedValue(mockLead);

      return Promise.resolve().then(() => {
        const statusField = element.shadowRoot.querySelector('#lead-status');
        expect(statusField.getAttribute('aria-label')).toContain('Lead status:');
        expect(statusField.getAttribute('aria-readonly')).toBe('true');
      });
    });
  });
});
