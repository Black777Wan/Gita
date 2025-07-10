import React, { useState, useRef, useEffect, useCallback } from 'react';
import { Play, Volume2 } from 'lucide-react';
import { useAppStore, Block } from '../store/appStore';

interface BlockEditorProps {
  block: Block;
  renderContent: (content: string) => React.ReactNode;
}

export const BlockEditor: React.FC<BlockEditorProps> = ({ block, renderContent }) => {
  const { updateBlockContent, deleteBlock, playAudioFromTimestamp } = useAppStore();
  const [isEditing, setIsEditing] = useState(false);
  const [content, setContent] = useState(block.content || '');
  const [isSaving, setIsSaving] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const saveTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    setContent(block.content || '');
  }, [block.content]);

  // Auto-save with debounce
  const debouncedSave = useCallback(async (newContent: string) => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }
    
    saveTimeoutRef.current = setTimeout(async () => {
      if (newContent.trim() === '') {
        // Don't auto-delete empty blocks, only on explicit save
        return;
      }
      
      if (newContent !== block.content) {
        try {
          setIsSaving(true);
          await updateBlockContent(block.id, newContent);
        } catch (error) {
          console.error('Auto-save failed:', error);
        } finally {
          setIsSaving(false);
        }
      }
    }, 500); // Save after 500ms of no typing
  }, [block.id, block.content, updateBlockContent]);

  // Clean up timeout on unmount
  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, []);

  const handleEdit = () => {
    setIsEditing(true);
    setTimeout(() => {
      textareaRef.current?.focus();
      textareaRef.current?.setSelectionRange(content.length, content.length);
    }, 0);
  };

  const handleSave = async () => {
    // Clear any pending auto-save
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
      saveTimeoutRef.current = null;
    }

    if (content.trim() === '') {
      // Delete empty block
      await deleteBlock(block.id);
      return;
    }

    if (content !== block.content) {
      try {
        setIsSaving(true);
        await updateBlockContent(block.id, content);
      } catch (error) {
        console.error('Save failed:', error);
        throw error;
      } finally {
        setIsSaving(false);
      }
    }
    setIsEditing(false);
  };

  const handleContentChange = (newContent: string) => {
    setContent(newContent);
    // Trigger auto-save
    debouncedSave(newContent);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      setContent(block.content || '');
      setIsEditing(false);
    }
  };

  const handlePlayAudio = () => {
    if (block.audio_timestamp) {
      playAudioFromTimestamp(block.audio_timestamp);
    }
  };

  const adjustTextareaHeight = () => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = textareaRef.current.scrollHeight + 'px';
    }
  };

  useEffect(() => {
    if (isEditing) {
      adjustTextareaHeight();
    }
  }, [isEditing, content]);

  return (
    <div className="block-editor">
      <div className="block-controls">
        <div className="block-bullet">•</div>
        {isSaving && (
          <div className="saving-indicator" title="Saving...">
            <span style={{ fontSize: '10px', color: '#6b7280' }}>●</span>
          </div>
        )}
        {block.audio_timestamp && (
          <button 
            className="audio-play-button"
            onClick={handlePlayAudio}
            title={`Play audio from ${block.audio_timestamp.timestamp_seconds}s`}
          >
            <Play size={12} />
          </button>
        )}
      </div>
      
      <div className="block-content">
        {isEditing ? (
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => handleContentChange(e.target.value)}
            onKeyDown={handleKeyDown}
            onBlur={handleSave}
            onInput={adjustTextareaHeight}
            className="block-input editing"
            rows={1}
            style={{
              minHeight: '24px',
              resize: 'none',
              overflow: 'hidden',
            }}
          />
        ) : (
          <div 
            className="block-display"
            onClick={handleEdit}
          >
            {content ? renderContent(content) : (
              <span className="empty-block">Click to edit...</span>
            )}
          </div>
        )}
        
        {block.audio_timestamp && (
          <div className="audio-indicator">
            <Volume2 size={12} />
            <span className="timestamp">
              {Math.floor(block.audio_timestamp.timestamp_seconds / 60)}:
              {(block.audio_timestamp.timestamp_seconds % 60).toString().padStart(2, '0')}
            </span>
          </div>
        )}
      </div>
    </div>
  );
};

