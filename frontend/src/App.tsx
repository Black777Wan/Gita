import React, { useEffect } from 'react';
import { useAppStore } from './store/appStore';
import { Sidebar } from './components/Sidebar';
import { MainEditor } from './components/MainEditor';
import { AudioControls } from './components/AudioControls';
import { format } from 'date-fns';
import './App.css';

function App() {
  const { 
    currentPage, 
    loadDailyNote, 
    isLoading,
    audioState,
    pendingSaves,
    waitForPendingSaves
  } = useAppStore();

  useEffect(() => {
    // Load today's daily note on startup
    const today = format(new Date(), 'yyyy-MM-dd');
    loadDailyNote(today);
  }, [loadDailyNote]);

  useEffect(() => {
    // Add beforeunload handler to warn about unsaved changes
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (pendingSaves.size > 0) {
        e.preventDefault();
        e.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
        return 'You have unsaved changes. Are you sure you want to leave?';
      }
    };

    // Add unload handler to attempt to save pending changes
    const handleUnload = async () => {
      if (pendingSaves.size > 0) {
        try {
          // Try to wait for pending saves, but don't block indefinitely
          await Promise.race([
            waitForPendingSaves(),
            new Promise(resolve => setTimeout(resolve, 1000)) // 1 second timeout
          ]);
        } catch (error) {
          console.error('Failed to save pending changes on unload:', error);
        }
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    window.addEventListener('unload', handleUnload);
    
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
      window.removeEventListener('unload', handleUnload);
    };
  }, [pendingSaves, waitForPendingSaves]);

  return (
    <div className="app">
      <div className="app-header">
        <div className="app-title">
          <h1>Gita</h1>
          <span className="subtitle">Research & Audio Note-Taking</span>
          {pendingSaves.size > 0 && (
            <span className="pending-saves-indicator" title={`${pendingSaves.size} unsaved changes`}>
              ● {pendingSaves.size} unsaved
            </span>
          )}
        </div>
        <AudioControls />
      </div>
      
      <div className="app-body">
        <Sidebar />
        <div className="main-content">
          {isLoading ? (
            <div className="loading">Loading...</div>
          ) : (
            <MainEditor />
          )}
        </div>
      </div>
      
      {audioState.isRecording && (
        <div className="recording-indicator">
          <div className="recording-dot"></div>
          <span>Recording...</span>
        </div>
      )}
    </div>
  );
}

export default App;

