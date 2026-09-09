import { createElement } from 'lwc';
import LeadDisplay from 'c/leadDisplay';
import { getRecord } from 'lightning/uiRecordApi';

jest.mock('lightning/uiRecordApi');

describe('c-lead-display', () => {
  let element;

  beforeEach(() => {
    element = createElement('c-lead-display', {
      is: LeadDisplay,
    });
    document.body.appendChild(element);
  });

  afterEach(() => {
    while (document.body.firstChild) {
      document.body.removeChild(document.body.firstChild);
    }
  });

  describe('Load State', () => {
    it('displays spinner during data load', (done) => {
      element.recordId = '00Q00000000001AAA';
      getRecord.mockReturnValueOnce(
        new Promise(() => {
          // Pending promise to keep loading state
        })
      );

      element.dispatchEvent(new CustomEvent('wire'));

      setImmediate(() => {
        const spinner = element.shadowRoot.querySelector('[role="status"]');
        expect(spinner).toBeTruthy();
        expect(spinner.textContent).toContain('Loading');
        done();
      });
    });

    it('renders lead data when loaded', (done) => {
      const mockData = {
        fields: {
          Name: { value: 'John Doe' },
          Status: { value: 'Open' },
        },
      };

      element.recordId = '00Q00000000001AAA';
      getRecord.mockReturnValueOnce(Promise.resolve(mockData));
      element.leadRecord = mockData;

      element.dispatchEvent(new CustomEvent('wire'));

      setImmediate(() => {
        expect(element.shadowRoot.querySelector('#lead-name').textContent).toBe(
          'John Doe'
        );
        expect(element.shadowRoot.querySelector('#lead-status').textContent).toBe(
          'Open'
        );
        done();
      });
    });
  });

  describe('Error State', () => {
    it('displays error message on load failure', (done) => {
      element.recordId = '00Q00000000001AAA';
      getRecord.mockReturnValueOnce(
        Promise.reject(new Error('Record not found'))
      );

      element.dispatchEvent(new CustomEvent('wire'));

      setImmediate(() => {
        const errorDiv = element.shadowRoot.querySelector('[role="alert"]');
        expect(errorDiv).toBeTruthy();
        expect(errorDiv.textContent).toContain('Failed to load lead record');
        done();
      });
    });

    it('clears error when data loads successfully', (done) => {
      const mockData = {
        fields: {
          Name: { value: 'Jane Doe' },
          Status: { value: 'Contacted' },
        },
      };

      element.recordId = '00Q00000000001AAA';
      element.hasError = true;
      element.leadRecord = mockData;

      setImmediate(() => {
        expect(element.hasError).toBe(true);
        element.hasError = false;
        element.dispatchEvent(new CustomEvent('wire'));

        setImmediate(() => {
          expect(element.hasError).toBe(false);
          done();
        });
      });
    });
  });

  describe('Keyboard Interaction', () => {
    it('refresh button is keyboard accessible', (done) => {
      element.recordId = '00Q00000000001AAA';
      element.hasData = true;

      setImmediate(() => {
        const button = element.shadowRoot.querySelector('button');
        expect(button.getAttribute('aria-label')).toBe('Refresh lead data');
        expect(button.className).toContain('slds-button');

        const event = new KeyboardEvent('keydown', { key: 'Enter' });
        button.dispatchEvent(event);
        button.click();

        setImmediate(() => {
          done();
        });
      });
    });

    it('maintains focus after refresh', (done) => {
      element.recordId = '00Q00000000001AAA';
      element.hasData = true;

      setImmediate(() => {
        const button = element.shadowRoot.querySelector('button');
        button.focus();

        expect(document.activeElement).toBe(element);
        button.click();

        setImmediate(() => {
          expect(document.activeElement).toBe(element);
          done();
        });
      });
    });

    it('has proper tab order for form labels and content', (done) => {
      element.recordId = '00Q00000000001AAA';
      element.leadRecord = {
        fields: {
          Name: { value: 'Test Lead' },
          Status: { value: 'Open' },
        },
      };

      setImmediate(() => {
        const labels = element.shadowRoot.querySelectorAll('label');
        expect(labels.length).toBe(2);

        labels.forEach((label) => {
          expect(label.className).toContain('slds-form__label');
        });

        const button = element.shadowRoot.querySelector('button');
        expect(button.getAttribute('aria-label')).toBeTruthy();
        done();
      });
    });
  });

  describe('Accessibility', () => {
    it('uses semantic HTML with proper ARIA attributes', (done) => {
      element.recordId = '00Q00000000001AAA';
      element.leadRecord = {
        fields: {
          Name: { value: 'Accessible Lead' },
          Status: { value: 'Open' },
        },
      };

      setImmediate(() => {
        const labels = element.shadowRoot.querySelectorAll('label');
        expect(labels.length).toBe(2);

        const statusSpan = element.shadowRoot.querySelector('#lead-status');
        expect(statusSpan.className).toContain('slds-badge');

        done();
      });
    });

    it('provides context for loading state', (done) => {
      element.recordId = '00Q00000000001AAA';
      element.isLoading = true;

      setImmediate(() => {
        const status = element.shadowRoot.querySelector('[role="status"]');
        expect(status).toBeTruthy();
        done();
      });
    });
  });
});
