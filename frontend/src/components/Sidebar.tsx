import React, { useState } from 'react';
import { Calendar, Search, Plus, FileText } from 'lucide-react';
import { useAppStore } from '../store/appStore';
import { format, subDays } from 'date-fns';

export const Sidebar: React.FC = () => {
  const { loadDailyNote, loadPage, currentPage } = useAppStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [newPageTitle, setNewPageTitle] = useState('');
  const [showNewPageInput, setShowNewPageInput] = useState(false);

  const handleDailyNoteClick = (date: Date) => {
    const dateStr = format(date, 'yyyy-MM-dd');
    loadDailyNote(dateStr);
  };

  const handleCreatePage = async (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && newPageTitle.trim()) {
      await loadPage(newPageTitle.trim());
      setNewPageTitle('');
      setShowNewPageInput(false);
    } else if (e.key === 'Escape') {
      setNewPageTitle('');
      setShowNewPageInput(false);
    }
  };

  const recentDates = Array.from({ length: 7 }, (_, i) => subDays(new Date(), i));

  return (
    <div className="sidebar">
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
          >
            <Plus size={14} />
          </button>
        </div>
        
        {showNewPageInput && (
          <input
            type="text"
            placeholder="Page title..."
            value={newPageTitle}
            onChange={(e) => setNewPageTitle(e.target.value)}
            onKeyDown={handleCreatePage}
            onBlur={() => {
              if (!newPageTitle.trim()) {
                setShowNewPageInput(false);
              }
            }}
            className="new-page-input"
            autoFocus
          />
        )}
        
        <div className="pages-list">
          {/* In a full implementation, this would show recent/favorite pages */}
          <div className="empty-state">
            <span>Create pages by typing [[Page Name]] in your notes</span>
          </div>
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

