import React, { useState, useRef, useEffect, useCallback, useMemo } from 'react';
import { Play, Volume2 } from 'lucide-react';
import { useAppStore, Block } from '../store/appStore';
import debounce from 'lodash.debounce';

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
  const blockRef = useRef(block);
  
  // Keep block ref updated
  useEffect(() => {
    blockRef.current = block;
  }, [block]);

  useEffect(() => {
    setContent(block.content || '');
  }, [block.content]);

  const updateBlockContentRef = useRef(updateBlockContent);
  const deleteBlockRef = useRef(deleteBlock);
  const playAudioFromTimestampRef = useRef(playAudioFromTimestamp);
  
  // Keep function refs updated
  useEffect(() => {
    updateBlockContentRef.current = updateBlockContent;
    deleteBlockRef.current = deleteBlock;
    playAudioFromTimestampRef.current = playAudioFromTimestamp;
  }, [updateBlockContent, deleteBlock, playAudioFromTimestamp]);

  const performSave = useCallback(async (newContent: string) => {
    // Get the current block content to compare against
    const currentBlockContent = blockRef.current.content || '';
    
    if (newContent.trim() === '' && newContent !== currentBlockContent) {
      // If content became empty, and it wasn't already empty, it will be handled by explicit save/delete
      // This prevents auto-deleting blocks while typing if user temporarily clears content
      return;
    }
    if (newContent === currentBlockContent) {
      return; // No change, no need to save
    }

    try {
      setIsSaving(true);
      await updateBlockContentRef.current(blockRef.current.id, newContent);
      console.log(`Auto-saved block ${blockRef.current.id} with content: "${newContent}"`);
    } catch (error) {
      console.error('Auto-save failed:', error);
      // Optionally, revert content or notify user
    } finally {
      setIsSaving(false);
    }
  }, []); // No dependencies - all accessed via refs

  const debouncedSave = useMemo(() => {
    return debounce(performSave, 300); // Save after 300ms of no typing
  }, [performSave]);


  // Clean up timeout on unmount and ensure any pending saves are handled
  useEffect(() => {
    return () => {
      // Cancel any future invocations on unmount
      debouncedSave.cancel();
    };
  }, [debouncedSave]);

  const handleEdit = () => {
    setIsEditing(true);
    setTimeout(() => {
      textareaRef.current?.focus();
      textareaRef.current?.setSelectionRange(content.length, content.length);
    }, 0);
  };

  const handleSave = async () => {
    // Immediately trigger and flush any pending debounced save.
    debouncedSave.flush();
    debouncedSave.cancel(); // Cancel subsequent automatic saves.

    if (content.trim() === '') {
      try {
        await deleteBlockRef.current(blockRef.current.id);
      } catch (error) {
        console.error('Failed to delete block:', error);
      }
      return;
    }

    // Since flush() is async in its effect, we might need to manually save if content is different
    // and the debounced function hasn't fired yet.
    if (content !== blockRef.current.content) {
      try {
        setIsSaving(true);
        await updateBlockContentRef.current(blockRef.current.id, content);
        console.log(`Manually saved block ${blockRef.current.id} with content: "${content}"`);
      } catch (error) {
        console.error('Save failed:', error);
      } finally {
        setIsSaving(false);
      }
    }
    setIsEditing(false);
  };

  const handleBlur = () => {
    // Force save on blur to prevent data loss during navigation
    if (content !== blockRef.current.content) {
      debouncedSave.flush();
    }
    setIsEditing(false);
  };

  const handleContentChange = useCallback((newContent: string) => {
    setContent(newContent);
    // Only call debounced save if content actually changed from the original block content
    if (newContent !== blockRef.current.content) {
      debouncedSave(newContent);
    }
  }, [debouncedSave]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      debouncedSave.cancel(); // Cancel any pending save
      setContent(blockRef.current.content || '');
      setIsEditing(false);
    }
  };

  const handlePlayAudio = () => {
    if (blockRef.current.audio_timestamp) {
      playAudioFromTimestampRef.current(blockRef.current.audio_timestamp);
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
            onBlur={handleBlur}
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

