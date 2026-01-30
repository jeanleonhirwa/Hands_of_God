/**
 * React hook for MCP Desktop Tauri commands
 */

import { invoke } from '@tauri-apps/api/tauri';
import { useCallback } from 'react';
import { useAppStore } from '../store';

interface McpResponse {
  success: boolean;
  message: string;
  data?: unknown;
}

interface PromptResponse {
  message: string;
  tool_calls: Array<{
    id: string;
    name: string;
    arguments: Record<string, unknown>;
    status: string;
    dry_run_result?: unknown;
  }>;
  requires_approval: boolean;
}

export function useMcp() {
  const { setConnected, addMessage, setLoading, addAuditEntry, addPendingApproval } = useAppStore();

  const connect = useCallback(async (address?: string) => {
    try {
      const response = await invoke<McpResponse>('connect_mcp', { address });
      if (response.success) {
        setConnected(true);
        addAuditEntry({
          action: 'connect',
          service: 'system',
          details: response.message,
          result: 'success',
        });
      }
      return response;
    } catch (error) {
      console.error('Failed to connect:', error);
      throw error;
    }
  }, [setConnected, addAuditEntry]);

  const disconnect = useCallback(async () => {
    try {
      const response = await invoke<McpResponse>('disconnect_mcp');
      if (response.success) {
        setConnected(false);
      }
      return response;
    } catch (error) {
      console.error('Failed to disconnect:', error);
      throw error;
    }
  }, [setConnected]);

  const sendPrompt = useCallback(async (prompt: string) => {
    setLoading(true);
    addMessage({ role: 'user', content: prompt });

    try {
      const response = await invoke<PromptResponse>('send_prompt', { prompt });
      
      addMessage({
        role: 'assistant',
        content: response.message,
        toolCalls: response.tool_calls.map(tc => ({
          id: tc.id,
          name: tc.name,
          arguments: tc.arguments,
          status: tc.status as 'pending' | 'approved' | 'rejected' | 'executed',
        })),
      });

      // Add pending approvals
      if (response.requires_approval) {
        response.tool_calls
          .filter(tc => tc.status === 'pending')
          .forEach(tc => {
            addPendingApproval({
              id: tc.id,
              action: tc.name,
              description: JSON.stringify(tc.arguments),
              affectedPaths: [],
              dryRunResult: tc.dry_run_result,
            });
          });
      }

      return response;
    } catch (error) {
      addMessage({
        role: 'assistant',
        content: `Error: ${error instanceof Error ? error.message : 'Unknown error'}`,
      });
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, addMessage, addPendingApproval]);

  const approveAction = useCallback(async (toolCallId: string) => {
    try {
      const response = await invoke<McpResponse>('approve_action', { toolCallId });
      if (response.success) {
        addAuditEntry({
          action: 'approve',
          service: 'approval',
          details: `Approved action: ${toolCallId}`,
          result: 'success',
        });
      }
      return response;
    } catch (error) {
      console.error('Failed to approve:', error);
      throw error;
    }
  }, [addAuditEntry]);

  const rejectAction = useCallback(async (toolCallId: string) => {
    try {
      const response = await invoke<McpResponse>('reject_action', { toolCallId });
      addAuditEntry({
        action: 'reject',
        service: 'approval',
        details: `Rejected action: ${toolCallId}`,
        result: 'rejected',
      });
      return response;
    } catch (error) {
      console.error('Failed to reject:', error);
      throw error;
    }
  }, [addAuditEntry]);

  return {
    connect,
    disconnect,
    sendPrompt,
    approveAction,
    rejectAction,
  };
}
