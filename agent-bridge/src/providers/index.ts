/**
 * LLM Provider abstraction
 */

import { LLMConfig } from '../config';
import { logger } from '../utils/logger';

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  toolCallId?: string;
}

export interface ChatRequest {
  messages: ChatMessage[];
  tools?: unknown[];
  toolChoice?: 'auto' | 'none' | 'required';
}

export interface ChatResponse {
  content: string | null;
  toolCalls?: Array<{
    id: string;
    name: string;
    arguments: Record<string, unknown>;
  }>;
}

export interface LLMProvider {
  chat(request: ChatRequest): Promise<ChatResponse>;
}

class OpenAIProvider implements LLMProvider {
  private apiKey: string;
  private model: string;

  constructor(config: LLMConfig) {
    this.apiKey = config.apiKey || '';
    this.model = config.model;
  }

  async chat(request: ChatRequest): Promise<ChatResponse> {
    const { default: OpenAI } = await import('openai');
    const client = new OpenAI({ apiKey: this.apiKey });

    const response = await client.chat.completions.create({
      model: this.model,
      messages: request.messages as any,
      tools: request.tools as any,
      tool_choice: request.toolChoice as any
    });

    const choice = response.choices[0];
    return {
      content: choice.message.content,
      toolCalls: choice.message.tool_calls?.map(tc => ({
        id: tc.id,
        name: tc.function.name,
        arguments: JSON.parse(tc.function.arguments)
      }))
    };
  }
}

class AnthropicProvider implements LLMProvider {
  private apiKey: string;
  private model: string;

  constructor(config: LLMConfig) {
    this.apiKey = config.apiKey || '';
    this.model = config.model;
  }

  async chat(request: ChatRequest): Promise<ChatResponse> {
    const { default: Anthropic } = await import('@anthropic-ai/sdk');
    const client = new Anthropic({ apiKey: this.apiKey });

    const systemMessage = request.messages.find(m => m.role === 'system');
    const otherMessages = request.messages.filter(m => m.role !== 'system');

    const response = await client.messages.create({
      model: this.model,
      max_tokens: 4096,
      system: systemMessage?.content || '',
      messages: otherMessages.map(m => ({
        role: m.role as 'user' | 'assistant',
        content: m.content
      })),
      tools: request.tools as any
    });

    // Extract text content
    const textContent = response.content.find(c => c.type === 'text');
    const toolUseContent = response.content.filter(c => c.type === 'tool_use');

    return {
      content: textContent ? (textContent as any).text : null,
      toolCalls: toolUseContent.map((tc: any) => ({
        id: tc.id,
        name: tc.name,
        arguments: tc.input
      }))
    };
  }
}

class MockProvider implements LLMProvider {
  async chat(request: ChatRequest): Promise<ChatResponse> {
    logger.info('Mock provider processing request', { messageCount: request.messages.length });
    
    // Simple mock that returns a helpful message
    const userMessage = request.messages.find(m => m.role === 'user')?.content || '';
    
    // Check if the message asks for file operations
    if (userMessage.toLowerCase().includes('read') && userMessage.toLowerCase().includes('file')) {
      return {
        content: "I'll read that file for you.",
        toolCalls: [{
          id: 'mock-1',
          name: 'read_file',
          arguments: { path: './README.md' }
        }]
      };
    }

    if (userMessage.toLowerCase().includes('list') && userMessage.toLowerCase().includes('dir')) {
      return {
        content: "I'll list the directory contents.",
        toolCalls: [{
          id: 'mock-2',
          name: 'list_dir',
          arguments: { path: '.' }
        }]
      };
    }

    return {
      content: `Mock response to: "${userMessage.substring(0, 50)}..."\n\nI'm a mock LLM provider. Set LLM_PROVIDER=openai or LLM_PROVIDER=anthropic for real AI responses.`,
      toolCalls: []
    };
  }
}

export function createProvider(config: LLMConfig): LLMProvider {
  switch (config.provider) {
    case 'openai':
      return new OpenAIProvider(config);
    case 'anthropic':
      return new AnthropicProvider(config);
    case 'mock':
    default:
      return new MockProvider();
  }
}
