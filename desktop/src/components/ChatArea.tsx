import React, { useState, useRef, useEffect } from 'react';
import { clsx } from 'clsx';
import { useAppStore, Message } from '../store';
import { Send, Loader2, Bot, User, AlertCircle, CheckCircle } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark, oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';

export function ChatArea() {
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const { messages, addMessage, isLoading, setLoading, darkMode } = useAppStore();

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Auto-resize textarea
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = 'auto';
      inputRef.current.style.height = `${Math.min(inputRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage = input.trim();
    setInput('');
    addMessage({ role: 'user', content: userMessage });
    setLoading(true);

    // Simulate API call - in production this would call the agent bridge
    setTimeout(() => {
      addMessage({
        role: 'assistant',
        content: `I'll help you with that. Here's what I would do:\n\n\`\`\`bash\n# Example command\nnpm init -y\n\`\`\`\n\nThis action requires your approval. Would you like me to proceed?`,
        toolCalls: [
          {
            id: '1',
            name: 'run_command',
            arguments: { command: 'npm', args: ['init', '-y'] },
            status: 'pending',
          },
        ],
      });
      setLoading(false);
    }, 1000);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full pt-8">
      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-4 py-4">
        {messages.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="max-w-3xl mx-auto space-y-4">
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} darkMode={darkMode} />
            ))}
            {isLoading && <LoadingIndicator />}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Input Area */}
      <div className="border-t border-gray-5 dark:border-gray-4 p-4">
        <form onSubmit={handleSubmit} className="max-w-3xl mx-auto">
          <div className="relative">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ask me anything... (Shift+Enter for new line)"
              className={clsx(
                'w-full resize-none rounded-apple-lg px-4 py-3 pr-12',
                'bg-gray-6 dark:bg-gray-5 text-body',
                'placeholder:text-label-tertiary',
                'border border-transparent focus:border-system-blue',
                'focus:outline-none focus:ring-0',
                'transition-all duration-150 ease-apple-ease',
                'min-h-[48px] max-h-[200px]'
              )}
              rows={1}
              disabled={isLoading}
            />
            <button
              type="submit"
              disabled={!input.trim() || isLoading}
              className={clsx(
                'absolute right-2 bottom-2 p-2 rounded-apple',
                'transition-all duration-150 ease-apple-ease',
                input.trim() && !isLoading
                  ? 'bg-system-blue text-white hover:opacity-90'
                  : 'bg-gray-4 dark:bg-gray-3 text-label-tertiary cursor-not-allowed'
              )}
            >
              {isLoading ? (
                <Loader2 className="w-5 h-5 animate-spin" />
              ) : (
                <Send className="w-5 h-5" />
              )}
            </button>
          </div>
          <p className="text-caption-1 text-label-tertiary mt-2 text-center">
            MCP will ask for approval before executing any commands
          </p>
        </form>
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center max-w-md">
        <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-system-blue/10 flex items-center justify-center">
          <Bot className="w-8 h-8 text-system-blue" />
        </div>
        <h2 className="text-title-2 font-semibold mb-2">Welcome to MCP</h2>
        <p className="text-body text-label-secondary mb-6">
          Your AI-powered local development assistant. I can help you manage files, 
          run commands, control git, and more — all with your approval.
        </p>
        <div className="grid grid-cols-2 gap-2 text-left">
          <SuggestionCard text="Scaffold a new React app" />
          <SuggestionCard text="Show git status" />
          <SuggestionCard text="Run npm install" />
          <SuggestionCard text="Create a snapshot" />
        </div>
      </div>
    </div>
  );
}

function SuggestionCard({ text }: { text: string }) {
  const { addMessage } = useAppStore();
  
  return (
    <button
      onClick={() => addMessage({ role: 'user', content: text })}
      className={clsx(
        'p-3 rounded-apple text-left text-subhead',
        'bg-gray-6 dark:bg-gray-5',
        'hover:bg-gray-5 dark:hover:bg-gray-4',
        'transition-colors duration-150'
      )}
    >
      {text}
    </button>
  );
}

function MessageBubble({ message, darkMode }: { message: Message; darkMode: boolean }) {
  const isUser = message.role === 'user';

  return (
    <div className={clsx('flex gap-3', isUser && 'flex-row-reverse')}>
      {/* Avatar */}
      <div
        className={clsx(
          'w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0',
          isUser ? 'bg-system-blue' : 'bg-gray-4 dark:bg-gray-3'
        )}
      >
        {isUser ? (
          <User className="w-4 h-4 text-white" />
        ) : (
          <Bot className="w-4 h-4 text-label-primary" />
        )}
      </div>

      {/* Content */}
      <div
        className={clsx(
          'flex-1 max-w-[80%] rounded-apple-lg px-4 py-3',
          isUser
            ? 'bg-system-blue text-white'
            : 'bg-gray-6 dark:bg-gray-5 text-label-primary'
        )}
      >
        <div className={clsx('prose prose-sm max-w-none', isUser && 'prose-invert')}>
          <ReactMarkdown
            components={{
              code({ node, className, children, ...props }) {
                const match = /language-(\w+)/.exec(className || '');
                const isInline = !match;
                return isInline ? (
                  <code className="px-1 py-0.5 rounded bg-black/10 dark:bg-white/10" {...props}>
                    {children}
                  </code>
                ) : (
                  <SyntaxHighlighter
                    style={darkMode ? oneDark : oneLight}
                    language={match[1]}
                    PreTag="div"
                    className="rounded-apple text-sm"
                  >
                    {String(children).replace(/\n$/, '')}
                  </SyntaxHighlighter>
                );
              },
            }}
          >
            {message.content}
          </ReactMarkdown>
        </div>

        {/* Tool Calls */}
        {message.toolCalls && message.toolCalls.length > 0 && (
          <div className="mt-3 space-y-2">
            {message.toolCalls.map((tool) => (
              <ToolCallCard key={tool.id} tool={tool} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function ToolCallCard({ tool }: { tool: Message['toolCalls'][0] }) {
  const { showConfirmModal } = useAppStore();

  const handleApprove = () => {
    showConfirmModal({
      title: 'Approve Action',
      description: `Execute "${tool.name}" with the specified arguments?`,
      action: 'Approve & Execute',
      onConfirm: () => {
        // Execute the tool
        console.log('Executing tool:', tool);
      },
      onCancel: () => {},
    });
  };

  return (
    <div className="bg-black/5 dark:bg-white/5 rounded-apple p-3">
      <div className="flex items-center justify-between mb-2">
        <span className="text-footnote font-medium">{tool.name}</span>
        <StatusBadge status={tool.status} />
      </div>
      <pre className="text-caption-1 bg-black/5 dark:bg-white/5 rounded p-2 overflow-x-auto">
        {JSON.stringify(tool.arguments, null, 2)}
      </pre>
      {tool.status === 'pending' && (
        <div className="flex gap-2 mt-2">
          <button onClick={handleApprove} className="btn-primary text-caption-1 py-1">
            Approve
          </button>
          <button className="btn-secondary text-caption-1 py-1">Reject</button>
        </div>
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const config = {
    pending: { color: 'bg-system-orange/20 text-system-orange', icon: AlertCircle },
    approved: { color: 'bg-system-green/20 text-system-green', icon: CheckCircle },
    executed: { color: 'bg-system-blue/20 text-system-blue', icon: CheckCircle },
    rejected: { color: 'bg-system-red/20 text-system-red', icon: AlertCircle },
  }[status] || { color: 'bg-gray-4', icon: AlertCircle };

  const Icon = config.icon;

  return (
    <span className={clsx('px-2 py-0.5 rounded-full text-caption-2 flex items-center gap-1', config.color)}>
      <Icon className="w-3 h-3" />
      {status}
    </span>
  );
}

function LoadingIndicator() {
  return (
    <div className="flex gap-3">
      <div className="w-8 h-8 rounded-full bg-gray-4 dark:bg-gray-3 flex items-center justify-center">
        <Bot className="w-4 h-4 text-label-primary" />
      </div>
      <div className="bg-gray-6 dark:bg-gray-5 rounded-apple-lg px-4 py-3">
        <div className="flex items-center gap-2">
          <Loader2 className="w-4 h-4 animate-spin text-system-blue" />
          <span className="text-body text-label-secondary">Thinking...</span>
        </div>
      </div>
    </div>
  );
}
