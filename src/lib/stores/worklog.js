import { writable } from 'svelte/store';

/**
 * @typedef {Object} Issue
 * @property {string} key
 * @property {Object} fields
 * @property {string} fields.summary
 * @property {Object} fields.status
 * @property {string} fields.status.name
 * @property {Object} [fields.assignee]
 * @property {string} [fields.assignee.display_name]
 * @property {string} [fields.assignee.email_address]
 */

/**
 * @typedef {Object} WorkLogForm
 * @property {string} issueKey
 * @property {string} workDate
 * @property {string} timeAmount
 * @property {string} timeUnit
 * @property {string} description
 */

function createWorkLogStore() {
  /** @type {import('svelte/store').Writable<{assignedIssues: Issue[], form: WorkLogForm}>} */
  const { subscribe, set, update } = writable({
    assignedIssues: /** @type {Issue[]} */ ([]),
    form: {
      issueKey: '',
      workDate: new Date().toISOString().split('T')[0],
      timeAmount: '',
      timeUnit: 'h',
      description: ''
    }
  });

  return {
    subscribe,
    /**
     * @param {Issue[]} issues
     */
    setAssignedIssues: (issues) => update(state => ({
      ...state,
      assignedIssues: issues
    })),
    /**
     * @param {Partial<WorkLogForm>} formData
     */
    updateForm: (formData) => update(state => ({
      ...state,
      form: { ...state.form, ...formData }
    })),
    resetForm: () => update(state => ({
      ...state,
      form: {
        ...state.form,
        timeAmount: '',
        description: ''
      }
    })),
    clearForm: () => update(state => ({
      ...state,
      form: {
        issueKey: '',
        workDate: new Date().toISOString().split('T')[0],
        timeAmount: '',
        timeUnit: 'h',
        description: ''
      }
    }))
  };
}

export const workLogStore = createWorkLogStore();
