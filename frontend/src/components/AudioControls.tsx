import React, { useEffect } from 'react';
import { Mic, MicOff, Settings } from 'lucide-react';
import { useAppStore } from '../store/appStore';

export const AudioControls: React.FC = () => {
  const { 
    audioState, 
    currentPage,
    startRecording, 
    stopRecording, 
    loadAudioDevices 
  } = useAppStore();

  useEffect(() => {
    loadAudioDevices();
  }, [loadAudioDevices]);

  const handleToggleRecording = async () => {
    if (!currentPage) {
      alert('Please select a page before recording');
      return;
    }

    try {
      if (audioState.isRecording) {
        await stopRecording();
      } else {
        await startRecording(currentPage.id);
      }
    } catch (error) {
      console.error('Recording error:', error);
      alert('Failed to toggle recording. Please check your audio permissions.');
    }
  };

  const formatRecordingTime = () => {
    if (!audioState.startTime) return '00:00';
    
    const elapsed = Math.floor((Date.now() - audioState.startTime) / 1000);
    const minutes = Math.floor(elapsed / 60);
    const seconds = elapsed % 60;
    
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  };

  return (
    <div className="audio-controls">
      <div className="recording-info">
        {audioState.isRecording && (
          <div className="recording-time">
            {formatRecordingTime()}
          </div>
        )}
      </div>
      
      <button
        className={`record-button ${audioState.isRecording ? 'recording' : ''}`}
        onClick={handleToggleRecording}
        disabled={!currentPage}
        title={audioState.isRecording ? 'Stop Recording' : 'Start Recording'}
      >
        {audioState.isRecording ? (
          <MicOff size={20} />
        ) : (
          <Mic size={20} />
        )}
        <span className="button-text">
          {audioState.isRecording ? 'Stop' : 'Record'}
        </span>
      </button>

      <div className="audio-devices">
        <button 
          className="settings-button"
          title="Audio Settings"
        >
          <Settings size={16} />
        </button>
        
        <div className="device-status">
          <div className="device-count">
            {audioState.devices.filter(d => d.device_type === 'input').length} mic(s)
          </div>
        </div>
      </div>
    </div>
  );
};

