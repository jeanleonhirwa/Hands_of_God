import React from 'react';
import { clsx } from 'clsx';
import { useAppStore } from '../store';
import { X, FileText, Terminal, GitBranch, Camera, CheckCircle, XCircle, Clock } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

const serviceIcons: Record<string, React.ElementType> = {
  file: FileText,
  command: Terminal,
  git: GitBranch,
  snapshot: Camera,
};

export function ActivityPanel() {
  const { toggleActivityPanel, auditLog, pendingApprovals } = useAppStore();

  return (
    <aside className="w-80 border-l border-gray-5 dark:border-gray-4 bg-bg-secondary flex flex-col pt-8">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-5 dark:border-gray-4">
        <h2 className="text-headline font-semibold">Activity</h2>
        <button
          onClick={toggleActivityPanel}
          className="p-1.5 rounded-apple hover:bg-gray-5 dark:hover:bg-gray-4 transition-colors"
        >
          <X className="w-4 h-4 text-label-secondary" />
        </button>
      </div>

      {/* Pending Approvals */}
      {pendingApprovals.length > 0 && (
        <div className="px-4 py-3 border-b border-gray-5 dark:border-gray-4">
          <h3 className="text-caption-1 text-label-secondary uppercase tracking-wide mb-2">
            Pending Approvals ({pendingApprovals.length})
          </h3>
          <div className="space-y-2">
            {pendingApprovals.map((approval) => (
              <PendingApprovalCard key={approval.id} approval={approval} />
            ))}
          </div>
        </div>
      )}

      {/* Audit Log */}
      <div className="flex-1 overflow-y-auto">
        <div className="px-4 py-3">
          <h3 className="text-caption-1 text-label-secondary uppercase tracking-wide mb-2">
            Recent Activity
          </h3>
          {auditLog.length === 0 ? (
            <p className="text-subhead text-label-tertiary text-center py-8">
              No activity yet
            </p>
          ) : (
            <div className="space-y-1">
              {auditLog.map((entry) => (
                <AuditLogEntry key={entry.id} entry={entry} />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Footer Stats */}
      <div className="px-4 py-3 border-t border-gray-5 dark:border-gray-4">
        <div className="grid grid-cols-3 gap-2 text-center">
          <StatItem label="Actions" value={auditLog.length} />
          <StatItem
            label="Success"
            value={auditLog.filter((e) => e.result === 'success').length}
          />
          <StatItem
            label="Pending"
            value={pendingApprovals.length}
          />
        </div>
      </div>
    </aside>
  );
}

function PendingApprovalCard({ approval }: { approval: any }) {
  const { removePendingApproval, showConfirmModal, addAuditEntry } = useAppStore();

  const handleApprove = () => {
    showConfirmModal({
      title: 'Approve Action',
      description: approval.description,
      action: 'Approve & Execute',
      onConfirm: () => {
        addAuditEntry({
          action: approval.action,
          service: 'approval',
          details: approval.description,
          result: 'success',
        });
        removePendingApproval(approval.id);
      },
      onCancel: () => {},
    });
  };

  const handleReject = () => {
    addAuditEntry({
      action: approval.action,
      service: 'approval',
      details: `Rejected: ${approval.description}`,
      result: 'rejected',
    });
    removePendingApproval(approval.id);
  };

  return (
    <div className="bg-system-orange/10 border border-system-orange/30 rounded-apple p-3">
      <div className="flex items-start gap-2">
        <Clock className="w-4 h-4 text-system-orange mt-0.5" />
        <div className="flex-1 min-w-0">
          <p className="text-subhead font-medium truncate">{approval.action}</p>
          <p className="text-caption-1 text-label-secondary truncate">
            {approval.description}
          </p>
          <div className="flex gap-2 mt-2">
            <button
              onClick={handleApprove}
              className="text-caption-1 px-2 py-1 bg-system-green text-white rounded-apple-sm"
            >
              Approve
            </button>
            <button
              onClick={handleReject}
              className="text-caption-1 px-2 py-1 bg-gray-4 dark:bg-gray-3 rounded-apple-sm"
            >
              Reject
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function AuditLogEntry({ entry }: { entry: any }) {
  const Icon = serviceIcons[entry.service] || FileText;
  const isSuccess = entry.result === 'success';

  return (
    <div
      className={clsx(
        'flex items-start gap-3 p-2 rounded-apple',
        'hover:bg-gray-6 dark:hover:bg-gray-5 transition-colors'
      )}
    >
      <div
        className={clsx(
          'w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0',
          isSuccess ? 'bg-system-green/10' : 'bg-system-red/10'
        )}
      >
        <Icon
          className={clsx(
            'w-4 h-4',
            isSuccess ? 'text-system-green' : 'text-system-red'
          )}
        />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-subhead font-medium truncate">{entry.action}</span>
          {isSuccess ? (
            <CheckCircle className="w-3 h-3 text-system-green flex-shrink-0" />
          ) : (
            <XCircle className="w-3 h-3 text-system-red flex-shrink-0" />
          )}
        </div>
        <p className="text-caption-1 text-label-secondary truncate">{entry.details}</p>
        <p className="text-caption-2 text-label-tertiary">
          {formatDistanceToNow(entry.timestamp, { addSuffix: true })}
        </p>
      </div>
    </div>
  );
}

function StatItem({ label, value }: { label: string; value: number }) {
  return (
    <div>
      <div className="text-title-3 font-semibold">{value}</div>
      <div className="text-caption-2 text-label-secondary">{label}</div>
    </div>
  );
}
