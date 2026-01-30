import React, { useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { ChatArea } from './components/ChatArea';
import { ActivityPanel } from './components/ActivityPanel';
import { ConfirmModal } from './components/ConfirmModal';
import { useAppStore } from './store';

function App() {
  const { darkMode, setDarkMode, showActivityPanel } = useAppStore();

  // Detect system dark mode preference
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    setDarkMode(mediaQuery.matches);

    const handler = (e: MediaQueryListEvent) => setDarkMode(e.matches);
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, [setDarkMode]);

  // Apply dark mode class
  useEffect(() => {
    document.documentElement.classList.toggle('dark', darkMode);
  }, [darkMode]);

  return (
    <div className="h-screen w-screen flex overflow-hidden bg-bg-primary">
      {/* Titlebar drag region for Tauri */}
      <div 
        data-tauri-drag-region 
        className="fixed top-0 left-0 right-0 h-8 z-50"
      />

      {/* Left Sidebar - Tools & History */}
      <Sidebar />

      {/* Main Chat Area */}
      <main className="flex-1 flex flex-col min-w-0">
        <ChatArea />
      </main>

      {/* Right Panel - Activity & Audit Log */}
      {showActivityPanel && <ActivityPanel />}

      {/* Modals */}
      <ConfirmModal />
    </div>
  );
}

export default App;
