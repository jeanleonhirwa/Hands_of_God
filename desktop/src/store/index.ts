import { create } from 'zustand';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  toolCalls?: ToolCall[];
  toolResults?: ToolResult[];
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  status: 'pending' | 'approved' | 'rejected' | 'executed';
}

export interface ToolResult {
  toolCallId: string;
  name: string;
  result?: unknown;
  error?: string;
  dryRun: boolean;
}

export interface AuditEntry {
  id: string;
  timestamp: Date;
  action: string;
  service: string;
  details: string;
  result: string;
}

export interface PendingApproval {
  id: string;
  action: string;
  description: string;
  affectedPaths: string[];
  dryRunResult?: unknown;
}

interface AppState {
  // UI State
  darkMode: boolean;
  setDarkMode: (dark: boolean) => void;
  showActivityPanel: boolean;
  toggleActivityPanel: () => void;
  activeTab: 'chat' | 'tools' | 'history';
  setActiveTab: (tab: 'chat' | 'tools' | 'history') => void;

  // Chat State
  messages: Message[];
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void;
  clearMessages: () => void;
  isLoading: boolean;
  setLoading: (loading: boolean) => void;

  // Tool State
  pendingApprovals: PendingApproval[];
  addPendingApproval: (approval: PendingApproval) => void;
  removePendingApproval: (id: string) => void;
  clearPendingApprovals: () => void;

  // Audit Log
  auditLog: AuditEntry[];
  addAuditEntry: (entry: Omit<AuditEntry, 'id' | 'timestamp'>) => void;

  // Confirm Modal
  confirmModal: {
    open: boolean;
    title: string;
    description: string;
    action: string;
    onConfirm: () => void;
    onCancel: () => void;
    destructive?: boolean;
  } | null;
  showConfirmModal: (config: Omit<NonNullable<AppState['confirmModal']>, 'open'>) => void;
  hideConfirmModal: () => void;

  // MCP Connection
  connected: boolean;
  setConnected: (connected: boolean) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  // UI State
  darkMode: false,
  setDarkMode: (dark) => set({ darkMode: dark }),
  showActivityPanel: true,
  toggleActivityPanel: () => set((s) => ({ showActivityPanel: !s.showActivityPanel })),
  activeTab: 'chat',
  setActiveTab: (tab) => set({ activeTab: tab }),

  // Chat State
  messages: [],
  addMessage: (message) =>
    set((s) => ({
      messages: [
        ...s.messages,
        {
          ...message,
          id: crypto.randomUUID(),
          timestamp: new Date(),
        },
      ],
    })),
  clearMessages: () => set({ messages: [] }),
  isLoading: false,
  setLoading: (loading) => set({ isLoading: loading }),

  // Tool State
  pendingApprovals: [],
  addPendingApproval: (approval) =>
    set((s) => ({ pendingApprovals: [...s.pendingApprovals, approval] })),
  removePendingApproval: (id) =>
    set((s) => ({
      pendingApprovals: s.pendingApprovals.filter((a) => a.id !== id),
    })),
  clearPendingApprovals: () => set({ pendingApprovals: [] }),

  // Audit Log
  auditLog: [],
  addAuditEntry: (entry) =>
    set((s) => ({
      auditLog: [
        { ...entry, id: crypto.randomUUID(), timestamp: new Date() },
        ...s.auditLog,
      ].slice(0, 100), // Keep last 100 entries
    })),

  // Confirm Modal
  confirmModal: null,
  showConfirmModal: (config) =>
    set({ confirmModal: { ...config, open: true } }),
  hideConfirmModal: () => set({ confirmModal: null }),

  // MCP Connection
  connected: false,
  setConnected: (connected) => set({ connected }),
}));
