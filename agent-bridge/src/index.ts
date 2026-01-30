/**
 * MCP Agent Bridge
 * 
 * Connects LLM providers (OpenAI, Anthropic, local models) to the MCP Core Server.
 * Handles prompt processing, tool calling, and approval flows.
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { config } from 'dotenv';
import { logger } from './utils/logger';
import { LLMProvider, createProvider } from './providers';
import { MCPClient } from './client';
import { AgentConfig, loadConfig } from './config';

config(); // Load .env

export class AgentBridge {
  private config: AgentConfig;
  private mcpClient: MCPClient;
  private llmProvider: LLMProvider;

  constructor(agentConfig: AgentConfig) {
    this.config = agentConfig;
    this.mcpClient = new MCPClient(agentConfig.mcpServerAddress);
    this.llmProvider = createProvider(agentConfig.llm);
  }

  /**
   * Process a user prompt through the LLM and execute any tool calls
   */
  async processPrompt(prompt: string): Promise<AgentResponse> {
    logger.info('Processing prompt', { promptLength: prompt.length });

    // Get available tools from MCP
    const tools = await this.getAvailableTools();

    // Send to LLM with tool definitions
    const llmResponse = await this.llmProvider.chat({
      messages: [
        { role: 'system', content: this.getSystemPrompt() },
        { role: 'user', content: prompt }
      ],
      tools,
      toolChoice: 'auto'
    });

    // If LLM wants to call tools
    if (llmResponse.toolCalls && llmResponse.toolCalls.length > 0) {
      const toolResults = await this.executeToolCalls(llmResponse.toolCalls);
      
      return {
        message: llmResponse.content || '',
        toolCalls: llmResponse.toolCalls,
        toolResults,
        requiresApproval: toolResults.some(r => r.requiresApproval)
      };
    }

    return {
      message: llmResponse.content || '',
      toolCalls: [],
      toolResults: []
    };
  }

  /**
   * Execute tool calls through MCP with dry-run support
   */
  private async executeToolCalls(toolCalls: ToolCall[]): Promise<ToolResult[]> {
    const results: ToolResult[] = [];

    for (const call of toolCalls) {
      logger.info('Executing tool call', { tool: call.name });

      try {
        // Always do dry-run first unless auto-approved
        const dryRunResult = await this.mcpClient.executeTool(call.name, call.arguments, true);
        
        results.push({
          toolCallId: call.id,
          name: call.name,
          dryRunResult,
          requiresApproval: !this.isAutoApproved(call.name, call.arguments),
          executed: false
        });
      } catch (error) {
        results.push({
          toolCallId: call.id,
          name: call.name,
          error: error instanceof Error ? error.message : 'Unknown error',
          requiresApproval: false,
          executed: false
        });
      }
    }

    return results;
  }

  /**
   * Execute an approved tool call
   */
  async executeApproved(toolCallId: string, approvalToken: string): Promise<ToolResult> {
    // Implementation would look up the pending tool call and execute it
    logger.info('Executing approved tool call', { toolCallId });
    
    return {
      toolCallId,
      name: 'unknown',
      executed: true,
      requiresApproval: false
    };
  }

  /**
   * Get available tools in LLM-compatible format
   */
  private async getAvailableTools(): Promise<Tool[]> {
    return [
      {
        type: 'function',
        function: {
          name: 'read_file',
          description: 'Read the contents of a file',
          parameters: {
            type: 'object',
            properties: {
              path: { type: 'string', description: 'File path to read' }
            },
            required: ['path']
          }
        }
      },
      {
        type: 'function',
        function: {
          name: 'create_file',
          description: 'Create or overwrite a file with content',
          parameters: {
            type: 'object',
            properties: {
              path: { type: 'string', description: 'File path to create' },
              content: { type: 'string', description: 'File content' }
            },
            required: ['path', 'content']
          }
        }
      },
      {
        type: 'function',
        function: {
          name: 'run_command',
          description: 'Run a whitelisted command',
          parameters: {
            type: 'object',
            properties: {
              command: { type: 'string', description: 'Command to run' },
              args: { type: 'array', items: { type: 'string' }, description: 'Command arguments' },
              cwd: { type: 'string', description: 'Working directory' }
            },
            required: ['command']
          }
        }
      },
      {
        type: 'function',
        function: {
          name: 'git_status',
          description: 'Get git repository status',
          parameters: {
            type: 'object',
            properties: {
              repo_path: { type: 'string', description: 'Path to git repository' }
            },
            required: ['repo_path']
          }
        }
      },
      {
        type: 'function',
        function: {
          name: 'git_commit',
          description: 'Create a git commit',
          parameters: {
            type: 'object',
            properties: {
              repo_path: { type: 'string', description: 'Path to git repository' },
              message: { type: 'string', description: 'Commit message' },
              files: { type: 'array', items: { type: 'string' }, description: 'Files to commit' }
            },
            required: ['repo_path', 'message', 'files']
          }
        }
      },
      {
        type: 'function',
        function: {
          name: 'list_dir',
          description: 'List directory contents',
          parameters: {
            type: 'object',
            properties: {
              path: { type: 'string', description: 'Directory path' }
            },
            required: ['path']
          }
        }
      }
    ];
  }

  private getSystemPrompt(): string {
    return `You are an AI assistant with access to local development tools through MCP (Model Context Protocol).
You can read/write files, run commands, manage git repositories, and more.

IMPORTANT SAFETY RULES:
1. Always prefer dry-run mode first to preview changes
2. Never execute destructive operations without explicit user confirmation
3. Stay within allowed directories and whitelisted commands
4. Explain what you're about to do before doing it

When using tools:
- Use read_file to examine existing code
- Use create_file to create or modify files
- Use run_command for npm, git, cargo, etc.
- Use git_status and git_commit for version control

Always be helpful, safe, and transparent about your actions.`;
  }

  private isAutoApproved(toolName: string, args: Record<string, unknown>): boolean {
    // Read operations are auto-approved
    const readOnlyTools = ['read_file', 'list_dir', 'git_status', 'get_system_info'];
    return readOnlyTools.includes(toolName);
  }
}

// Type definitions
interface Tool {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}

interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

interface ToolResult {
  toolCallId: string;
  name: string;
  dryRunResult?: unknown;
  result?: unknown;
  error?: string;
  requiresApproval: boolean;
  executed: boolean;
}

interface AgentResponse {
  message: string;
  toolCalls: ToolCall[];
  toolResults: ToolResult[];
  requiresApproval?: boolean;
}

// Main entry point
async function main() {
  const agentConfig = loadConfig();
  const bridge = new AgentBridge(agentConfig);
  
  logger.info('MCP Agent Bridge started', { 
    provider: agentConfig.llm.provider,
    mcpServer: agentConfig.mcpServerAddress 
  });

  // In production, this would expose an API or connect to a message queue
  // For now, we just log that we're ready
  logger.info('Agent bridge ready for connections');
}

main().catch(err => {
  logger.error('Failed to start agent bridge', { error: err });
  process.exit(1);
});

export { AgentBridge, AgentResponse, ToolCall, ToolResult };
