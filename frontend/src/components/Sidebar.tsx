import React, { useState } from 'react';
import { Calendar, Search, Plus, FileText } from 'lucide-react';
import { useAppStore } from '../store/appStore';
import { format, subDays } from 'date-fns';

export const Sidebar: React.FC = () => {
  const { loadDailyNote, loadPage, currentPage, pages, isLoading, error } = useAppStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [newPageTitle, setNewPageTitle] = useState('');
  const [showNewPageInput, setShowNewPageInput] = useState(false);
  const [isCreatingPage, setIsCreatingPage] = useState(false);

  const handleDailyNoteClick = async (date: Date) => {
    try {
      const dateStr = format(date, 'yyyy-MM-dd');
      await loadDailyNote(dateStr);
    } catch (error) {
      console.error('Failed to load daily note:', error);
    }
  };

  const handlePageClick = async (title: string) => {
    try {
      await loadPage(title);
    } catch (error) {
      console.error('Failed to load page:', error);
    }
  };

  const handleCreatePage = async (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && newPageTitle.trim()) {
      setIsCreatingPage(true);
      try {
        await loadPage(newPageTitle.trim());
        setNewPageTitle('');
        setShowNewPageInput(false);
      } catch (error) {
        console.error('Failed to create page:', error);
      } finally {
        setIsCreatingPage(false);
      }
    } else if (e.key === 'Escape') {
      setNewPageTitle('');
      setShowNewPageInput(false);
    }
  };

  const recentDates = Array.from({ length: 7 }, (_, i) => subDays(new Date(), i));

  return (
    <div className="sidebar">
      {error && (
        <div className="error-message" style={{ 
          padding: '8px 12px', 
          margin: '8px 12px',
          backgroundColor: '#fee2e2',
          border: '1px solid #fecaca',
          borderRadius: '4px',
          color: '#dc2626',
          fontSize: '12px'
        }}>
          {error}
        </div>
      )}
      
      <div className="sidebar-section">
        <div className="section-header">
          <Search size={16} />
          <span>Search</span>
        </div>
        <input
          type="text"
          placeholder="Search pages..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />
      </div>

      <div className="sidebar-section">
        <div className="section-header">
          <Calendar size={16} />
          <span>Daily Notes</span>
        </div>
        <div className="daily-notes">
          {recentDates.map((date) => {
            const dateStr = format(date, 'yyyy-MM-dd');
            const displayStr = format(date, 'MMM d');
            const isToday = format(date, 'yyyy-MM-dd') === format(new Date(), 'yyyy-MM-dd');
            const isSelected = currentPage?.page_title === `Daily Notes/${dateStr}`;
            
            return (
              <button
                key={dateStr}
                className={`daily-note-item ${isSelected ? 'selected' : ''}`}
                onClick={() => handleDailyNoteClick(date)}
                disabled={isLoading}
              >
                <span className="date-display">
                  {isToday ? 'Today' : displayStr}
                </span>
                {isToday && <span className="today-indicator">•</span>}
              </button>
            );
          })}
        </div>
      </div>

      <div className="sidebar-section">
        <div className="section-header">
          <FileText size={16} />
          <span>Pages</span>
          <button
            className="add-page-button"
            onClick={() => setShowNewPageInput(true)}
            title="Create new page"
            disabled={isLoading || isCreatingPage}
          >
            <Plus size={14} />
          </button>
        </div>
        
        {showNewPageInput && (
          <input
            type="text"
            placeholder={isCreatingPage ? "Creating page..." : "Page title..."}
            value={newPageTitle}
            onChange={(e) => setNewPageTitle(e.target.value)}
            onKeyDown={handleCreatePage}
            onBlur={() => {
              if (!newPageTitle.trim() && !isCreatingPage) {
                setShowNewPageInput(false);
              }
            }}
            className="new-page-input"
            disabled={isCreatingPage}
            autoFocus
          />
        )}
        
        <div className="pages-list">
          {pages && pages.filter(p => !p.page_title?.startsWith("Daily Notes/")).map((page) => (
            <button
              key={page.id}
              className={`page-item ${currentPage?.id === page.id ? 'selected' : ''}`}
              onClick={() => handlePageClick(page.page_title!)}
              disabled={isLoading}
            >
              <FileText size={14} className="file-icon" />
              <span className="page-title">{page.page_title}</span>
            </button>
          ))}
          {(!pages || pages.filter(p => !p.page_title?.startsWith("Daily Notes/")).length === 0) && !showNewPageInput && (
             <div className="empty-state">
               <span>Create pages with [[Page Name]]</span>
             </div>
          )}
        </div>
      </div>

      <div className="sidebar-footer">
        <div className="app-info">
          <span className="version">Gita v0.1.0</span>
        </div>
      </div>
    </div>
  );
};

