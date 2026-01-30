import React from 'react';
import { clsx } from 'clsx';
import { useAppStore } from '../store';
import { AlertTriangle, X } from 'lucide-react';

export function ConfirmModal() {
  const { confirmModal, hideConfirmModal } = useAppStore();

  if (!confirmModal?.open) return null;

  const handleConfirm = () => {
    confirmModal.onConfirm();
    hideConfirmModal();
  };

  const handleCancel = () => {
    confirmModal.onCancel();
    hideConfirmModal();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={handleCancel}
      />

      {/* Modal */}
      <div
        className={clsx(
          'relative w-full max-w-md mx-4',
          'bg-bg-primary dark:bg-bg-tertiary rounded-apple-xl',
          'shadow-apple-modal animate-slide-up'
        )}
      >
        {/* Close button */}
        <button
          onClick={handleCancel}
          className={clsx(
            'absolute top-4 right-4 p-1.5 rounded-full',
            'hover:bg-gray-5 dark:hover:bg-gray-4 transition-colors'
          )}
        >
          <X className="w-5 h-5 text-label-secondary" />
        </button>

        {/* Content */}
        <div className="p-6">
          {/* Icon */}
          <div
            className={clsx(
              'w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center',
              confirmModal.destructive
                ? 'bg-system-red/10'
                : 'bg-system-blue/10'
            )}
          >
            <AlertTriangle
              className={clsx(
                'w-6 h-6',
                confirmModal.destructive ? 'text-system-red' : 'text-system-blue'
              )}
            />
          </div>

          {/* Title & Description */}
          <h2 className="text-title-3 font-semibold text-center mb-2">
            {confirmModal.title}
          </h2>
          <p className="text-body text-label-secondary text-center mb-6">
            {confirmModal.description}
          </p>

          {/* Actions */}
          <div className="flex gap-3">
            <button
              onClick={handleCancel}
              className="flex-1 btn-secondary"
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              className={clsx(
                'flex-1',
                confirmModal.destructive ? 'btn-destructive' : 'btn-primary'
              )}
            >
              {confirmModal.action}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
