import React from 'react';
import { clsx } from 'clsx';
import { useAppStore } from '../store';
import {
  MessageSquare,
  Wrench,
  History,
  FolderOpen,
  GitBranch,
  Terminal,
  Camera,
  Settings,
  Sun,
  Moon,
} from 'lucide-react';

const tools = [
  { id: 'files', name: 'Files', icon: FolderOpen, description: 'Read, write, manage files' },
  { id: 'git', name: 'Git', icon: GitBranch, description: 'Version control operations' },
  { id: 'command', name: 'Terminal', icon: Terminal, description: 'Run whitelisted commands' },
  { id: 'snapshot', name: 'Snapshots', icon: Camera, description: 'Backup & restore' },
];

export function Sidebar() {
  const { activeTab, setActiveTab, darkMode, setDarkMode, connected } = useAppStore();

  return (
    <aside className="sidebar w-64 pt-10">
      {/* Connection Status */}
      <div className="px-4 py-3 border-b border-gray-5 dark:border-gray-4">
        <div className="flex items-center gap-2">
          <div className={clsx(
            'w-2 h-2 rounded-full',
            connected ? 'bg-system-green' : 'bg-system-red'
          )} />
          <span className="text-footnote text-label-secondary">
            {connected ? 'MCP Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Navigation Tabs */}
      <nav className="p-2">
        <NavItem
          icon={MessageSquare}
          label="Chat"
          active={activeTab === 'chat'}
          onClick={() => setActiveTab('chat')}
        />
        <NavItem
          icon={Wrench}
          label="Tools"
          active={activeTab === 'tools'}
          onClick={() => setActiveTab('tools')}
        />
        <NavItem
          icon={History}
          label="History"
          active={activeTab === 'history'}
          onClick={() => setActiveTab('history')}
        />
      </nav>

      {/* Tools List (when tools tab active) */}
      {activeTab === 'tools' && (
        <div className="flex-1 overflow-y-auto px-2">
          <h3 className="text-caption-1 text-label-secondary uppercase tracking-wide px-3 py-2">
            Available Tools
          </h3>
          <div className="space-y-1">
            {tools.map((tool) => (
              <button
                key={tool.id}
                className={clsx(
                  'w-full flex items-center gap-3 px-3 py-2.5 rounded-apple',
                  'text-left transition-colors duration-150',
                  'hover:bg-gray-6 dark:hover:bg-gray-5'
                )}
              >
                <tool.icon className="w-5 h-5 text-system-blue" />
                <div className="flex-1 min-w-0">
                  <div className="text-subhead font-medium truncate">{tool.name}</div>
                  <div className="text-caption-1 text-label-secondary truncate">
                    {tool.description}
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Quick Actions */}
      {activeTab === 'chat' && (
        <div className="flex-1 overflow-y-auto px-2">
          <h3 className="text-caption-1 text-label-secondary uppercase tracking-wide px-3 py-2">
            Quick Actions
          </h3>
          <div className="space-y-1">
            <QuickAction label="Scaffold new project" />
            <QuickAction label="Run tests" />
            <QuickAction label="Git status" />
            <QuickAction label="Create snapshot" />
          </div>
        </div>
      )}

      {/* Footer */}
      <div className="p-2 border-t border-gray-5 dark:border-gray-4">
        <div className="flex items-center justify-between px-3">
          <button
            onClick={() => setDarkMode(!darkMode)}
            className={clsx(
              'p-2 rounded-apple transition-colors',
              'hover:bg-gray-6 dark:hover:bg-gray-5'
            )}
            title={darkMode ? 'Light mode' : 'Dark mode'}
          >
            {darkMode ? (
              <Sun className="w-5 h-5 text-label-secondary" />
            ) : (
              <Moon className="w-5 h-5 text-label-secondary" />
            )}
          </button>
          <button
            className={clsx(
              'p-2 rounded-apple transition-colors',
              'hover:bg-gray-6 dark:hover:bg-gray-5'
            )}
            title="Settings"
          >
            <Settings className="w-5 h-5 text-label-secondary" />
          </button>
        </div>
      </div>
    </aside>
  );
}

function NavItem({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: React.ElementType;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={clsx(
        'w-full flex items-center gap-3 px-3 py-2.5 rounded-apple',
        'transition-all duration-150 ease-apple-ease',
        active
          ? 'bg-system-blue/10 text-system-blue'
          : 'text-label-primary hover:bg-gray-6 dark:hover:bg-gray-5'
      )}
    >
      <Icon className="w-5 h-5" />
      <span className="text-subhead font-medium">{label}</span>
    </button>
  );
}

function QuickAction({ label }: { label: string }) {
  const { addMessage } = useAppStore();

  return (
    <button
      onClick={() => addMessage({ role: 'user', content: label })}
      className={clsx(
        'w-full text-left px-3 py-2 rounded-apple text-subhead',
        'text-system-blue hover:bg-system-blue/10',
        'transition-colors duration-150'
      )}
    >
      {label}
    </button>
  );
}
