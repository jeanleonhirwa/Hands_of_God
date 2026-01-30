/**
 * Configuration management for Agent Bridge
 */

export interface AgentConfig {
  mcpServerAddress: string;
  llm: LLMConfig;
  autoApprovePatterns: string[];
  maxTokens: number;
}

export interface LLMConfig {
  provider: 'openai' | 'anthropic' | 'local' | 'mock';
  apiKey?: string;
  endpoint?: string;
  model: string;
  temperature: number;
}

export function loadConfig(): AgentConfig {
  return {
    mcpServerAddress: process.env.MCP_SERVER_ADDRESS || 'localhost:50051',
    llm: {
      provider: (process.env.LLM_PROVIDER as LLMConfig['provider']) || 'mock',
      apiKey: process.env.LLM_API_KEY,
      endpoint: process.env.LLM_ENDPOINT,
      model: process.env.LLM_MODEL || 'gpt-4',
      temperature: parseFloat(process.env.LLM_TEMPERATURE || '0.7')
    },
    autoApprovePatterns: (process.env.AUTO_APPROVE_PATTERNS || 'read_file,list_dir,git_status').split(','),
    maxTokens: parseInt(process.env.MAX_TOKENS || '4096', 10)
  };
}
