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
    audioState 
  } = useAppStore();

  useEffect(() => {
    // Load today's daily note on startup
    const today = format(new Date(), 'yyyy-MM-dd');
    loadDailyNote(today);
  }, [loadDailyNote]);

  return (
    <div className="app">
      <div className="app-header">
        <div className="app-title">
          <h1>Gita</h1>
          <span className="subtitle">Research & Audio Note-Taking</span>
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

