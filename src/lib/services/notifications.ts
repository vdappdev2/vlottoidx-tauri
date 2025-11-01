import toast from 'svelte-5-french-toast';

/**
 * Show a success toast notification
 * @param message - The success message to display
 * @param details - Optional transaction or operation ID details
 */
export function showSuccess(message: string, details?: { txid?: string, operationId?: string }) {
  if (details?.txid) {
    toast.success(`${message}\nTx: ${details.txid}`, {
      duration: 4000,
      position: 'bottom-right',
      className: 'verusidx-toast'
    });
  } else if (details?.operationId) {
    toast.success(`${message}\nOp: ${details.operationId}`, {
      duration: 4000,
      position: 'bottom-right',
      className: 'verusidx-toast'
    });
  } else {
    toast.success(message, {
      duration: 4000,
      position: 'bottom-right',
      className: 'verusidx-toast'
    });
  }
}

/**
 * Show an error toast notification
 * @param message - The error message to display
 */
export function showError(message: string) {
  toast.error(message, {
    duration: 6000,
    position: 'bottom-right',
    className: 'verusidx-toast'
  });
}

/**
 * Show an info toast notification
 * @param message - The info message to display
 */
export function showInfo(message: string) {
  toast(message, {
    duration: 3000,
    position: 'bottom-right',
    className: 'verusidx-toast'
  });
}