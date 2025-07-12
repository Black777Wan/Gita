import React, { useState, useRef, useEffect } from 'react';
import { useAppStore } from '../store/appStore';
import { BlockEditor } from './BlockEditor';
import { format } from 'date-fns';

export const MainEditor: React.FC = () => {
  const { 
    blocks, 
    currentPage, 
    createBlock, 
    audioState 
  } = useAppStore();

  const [newBlockContent, setNewBlockContent] = useState('');
  const newBlockRef = useRef<HTMLTextAreaElement>(null);

  const pageBlocks = blocks.filter(block => !block.is_page);
  const sortedBlocks = pageBlocks.sort((a, b) => a.order - b.order);

  const handleCreateBlock = async (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      
      if (!newBlockContent.trim() || !currentPage) return;

      // Calculate audio metadata if recording
      let audioMeta;
      if (audioState.isRecording && audioState.recordingId && audioState.startTime) {
        const timestamp = Math.floor((Date.now() - audioState.startTime) / 1000);
        audioMeta = {
          recording_id: audioState.recordingId,
          timestamp,
        };
      }

      try {
        await createBlock({
          content: newBlockContent,
          parent_id: currentPage.id,
          order: sortedBlocks.length,
          is_page: false,
        }, audioMeta);

        setNewBlockContent('');
        newBlockRef.current?.focus();
      } catch (error) {
        console.error('Failed to create block:', error);
      }
    }
  };

  const handlePageTitleClick = async (title: string) => {
    // Navigate to linked page - wait for pending saves
    try {
      const { loadPage } = useAppStore.getState();
      await loadPage(title);
    } catch (error) {
      console.error('Failed to navigate to page:', title, error);
    }
  };

  // Save new block content on unmount if it exists
  useEffect(() => {
    return () => {
      if (newBlockContent.trim() && currentPage) {
        // Fire and forget, no need to await
        createBlock({
          content: newBlockContent,
          parent_id: currentPage.id,
          order: sortedBlocks.length,
          is_page: false,
        }).catch(error => {
          console.error('Failed to save new block on unmount:', error);
        });
      }
    };
  }, [newBlockContent, currentPage, createBlock, sortedBlocks.length]);

  const renderContent = (content: string) => {
    // Simple implementation of [[Page Link]] parsing
    const linkRegex = /\[\[([^\]]+)\]\]/g;
    const parts = content.split(linkRegex);
    
    return parts.map((part, index) => {
      if (index % 2 === 1) {
        // This is a page link
        return (
          <span 
            key={index}
            className="page-link"
            onClick={() => handlePageTitleClick(part)}
          >
            [[{part}]]
          </span>
        );
      }
      return part;
    });
  };

  if (!currentPage) {
    return (
      <div className="main-editor">
        <div className="no-page">
          <h2>No page selected</h2>
          <p>Create a daily note or navigate to a page to start editing.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="main-editor">
      <div className="page-header">
        <h1 className="page-title">
          {currentPage.page_title || 'Untitled Page'}
        </h1>
        <div className="page-meta">
          <span className="page-date">
            {format(new Date(currentPage.created_at), 'MMMM d, yyyy')}
          </span>
          {audioState.isRecording && (
            <span className="recording-badge">
              Recording Active
            </span>
          )}
        </div>
      </div>

      <div className="blocks-container">
        {sortedBlocks.map((block) => (
          <BlockEditor 
            key={block.id} 
            block={block}
            renderContent={renderContent}
          />
        ))}
        
        <div className="new-block">
          <div className="block-bullet">•</div>
          <textarea
            ref={newBlockRef}
            value={newBlockContent}
            onChange={(e) => setNewBlockContent(e.target.value)}
            onKeyDown={handleCreateBlock}
            placeholder="Start typing..."
            className="block-input"
            rows={1}
            style={{
              minHeight: '24px',
              resize: 'none',
              overflow: 'hidden',
            }}
            onInput={(e) => {
              const target = e.target as HTMLTextAreaElement;
              target.style.height = 'auto';
              target.style.height = target.scrollHeight + 'px';
            }}
          />
        </div>
      </div>
    </div>
  );
};

