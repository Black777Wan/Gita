import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Block {
  id: string;
  content?: string;
  parent_id?: string;
  order: number;
  is_page: boolean;
  page_title?: string;
  created_at: string;
  updated_at: string;
  audio_timestamp?: AudioTimestamp;
}

export interface AudioTimestamp {
  id: number;
  block_id: string;
  recording_id: string;
  timestamp_seconds: number;
  recording?: AudioRecording;
}

export interface AudioRecording {
  id: string;
  page_id: string;
  file_path: string;
  duration_seconds?: number;
  recorded_at: string;
}

export interface AudioDevice {
  name: string;
  is_default: boolean;
  device_type: string;
}

export interface AudioState {
  isRecording: boolean;
  recordingId?: string;
  pageId?: string;
  startTime?: number;
  devices: AudioDevice[];
}

export interface CreateBlockRequest {
  content?: string;
  parent_id?: string;
  order: number;
  is_page: boolean;
  page_title?: string;
}

export interface AudioMeta {
  recording_id: string;
  timestamp: number;
}

interface AppState {
  // Data state
  blocks: Block[];
  pages: Block[];
  currentPage?: Block;
  isLoading: boolean;
  error?: string;
  
  // Audio state
  audioState: AudioState;
  
  // Actions
  loadDailyNote: (date: string) => Promise<void>;
  loadPage: (title: string) => Promise<void>;
  createBlock: (blockData: CreateBlockRequest, audioMeta?: AudioMeta) => Promise<Block>;
  updateBlockContent: (blockId: string, content: string) => Promise<void>;
  deleteBlock: (blockId: string) => Promise<void>;
  
  // Audio actions
  startRecording: (pageId: string) => Promise<void>;
  stopRecording: () => Promise<void>;
  loadAudioDevices: () => Promise<void>;
  playAudioFromTimestamp: (audioTimestamp: AudioTimestamp) => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Initial state
  blocks: [],
  pages: [],
  currentPage: undefined,
  isLoading: false,
  error: undefined,
  audioState: {
    isRecording: false,
    devices: [],
  },

  // Data actions
  loadDailyNote: async (date: string) => {
    set({ isLoading: true, error: undefined });
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        const blocks: Block[] = await invoke('get_daily_note', { date });
        const currentPage = blocks.find(block => block.is_page);
        const pages: Block[] = await invoke('get_pages');
        set({
          blocks,
          currentPage,
          pages,
          isLoading: false
        });
      } catch (error) {
        console.error("Error invoking get_daily_note:", error);
        set({
          error: error as string,
          isLoading: false
        });
      }
    } else {
      console.warn('Tauri API not found. Using mock data for development.');
      // Create mock daily note page
      const mockPage: Block = {
        id: `daily-${date}`,
        content: undefined,
        page_title: `Daily Notes/${date}`,
        is_page: true,
        order: 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      
      // Get existing pages from state or create empty array
      const existingPages = get().pages || [];
      
      set({ 
        isLoading: false,
        blocks: [mockPage],
        currentPage: mockPage,
        pages: existingPages.some(p => p.id === mockPage.id) ? existingPages : [...existingPages, mockPage],
      });
    }
  },

  loadPage: async (title: string) => {
    set({ isLoading: true, error: undefined });
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        const page: Block | null = await invoke('get_page_by_title', { title });
        if (page) {
          const children: Block[] = await invoke('get_block_children', { parentId: page.id });
          const blocks = [page, ...children];
          const pages: Block[] = await invoke('get_pages');
          set({
            blocks,
            currentPage: page,
            pages,
            isLoading: false
          });
        } else {
          // Create new page
          const newPage = await get().createBlock({
            content: undefined,
            parent_id: undefined,
            order: 0,
            is_page: true,
            page_title: title,
          });
          
          if (newPage) {
            const pages: Block[] = await invoke('get_pages');
            set({
              blocks: [newPage],
              currentPage: newPage,
              pages,
              isLoading: false
            });
          } else {
            set({
              blocks: [],
              currentPage: undefined,
              isLoading: false,
              error: "Failed to create page: Tauri API not available."
            });
          }
        }
      } catch (error) {
        console.error("Error invoking get_page_by_title or get_block_children:", error);
        set({ 
          error: error as string,
          isLoading: false 
        });
      }
    } else {
      console.warn('Tauri API not found. Using mock data for development.');
      // Check if page already exists in state
      const existingPages = get().pages || [];
      let existingPage = existingPages.find(p => p.page_title === title);
      
      if (!existingPage) {
        // Create new mock page
        existingPage = {
          id: `page-${Date.now()}`,
          content: undefined,
          page_title: title,
          is_page: true,
          order: 0,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        
        // Add to pages list
        const updatedPages = [...existingPages, existingPage];
        set({
          pages: updatedPages,
          blocks: [existingPage],
          currentPage: existingPage,
          isLoading: false
        });
      } else {
        // Load existing page
        set({
          blocks: [existingPage],
          currentPage: existingPage,
          isLoading: false
        });
      }
    }
  },

  createBlock: async (blockData: CreateBlockRequest, audioMeta?: AudioMeta) => {
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        const newBlock: Block = await invoke('create_block', { blockData, audioMeta });

        set(state => ({
          blocks: [...state.blocks, newBlock]
        }));

        return newBlock;
      } catch (error) {
        console.error("Error invoking create_block:", error);
        set({ error: error as string });
        throw error;
      }
    } else {
      console.warn('Tauri API not found. Creating mock block for development.');
      // Create mock block
      const newBlock: Block = {
        id: `block-${Date.now()}`,
        content: blockData.content,
        parent_id: blockData.parent_id,
        order: blockData.order,
        is_page: blockData.is_page,
        page_title: blockData.page_title,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      
      // Add audio timestamp if provided
      if (audioMeta) {
        newBlock.audio_timestamp = {
          id: Date.now(),
          block_id: newBlock.id,
          recording_id: audioMeta.recording_id,
          timestamp_seconds: audioMeta.timestamp,
        };
      }

      set(state => ({
        blocks: [...state.blocks, newBlock],
        // Update pages if this is a page block
        pages: blockData.is_page ? [...(state.pages || []), newBlock] : state.pages
      }));

      return newBlock;
    }
  },

  updateBlockContent: async (blockId: string, content: string) => {
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        await invoke('update_block_content', { blockId, content });

        set(state => ({
          blocks: state.blocks.map(block =>
            block.id === blockId
              ? { ...block, content, updated_at: new Date().toISOString() }
              : block
          )
        }));
      } catch (error) {
        console.error("Error invoking update_block_content:", error);
        set({ error: error as string });
        throw error;
      }
    } else {
      console.warn('Tauri API not found. Updating mock block for development.');
      // Update mock block
      set(state => ({
        blocks: state.blocks.map(block =>
          block.id === blockId
            ? { ...block, content, updated_at: new Date().toISOString() }
            : block
        )
      }));
    }
  },

  deleteBlock: async (blockId: string) => {
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        await invoke('delete_block', { blockId });

        set(state => ({
          blocks: state.blocks.filter(block => block.id !== blockId)
        }));
      } catch (error) {
        console.error("Error invoking delete_block:", error);
        set({ error: error as string });
        throw error;
      }
    } else {
      console.warn('Tauri API not found. Deleting mock block for development.');
      // Delete mock block
      set(state => ({
        blocks: state.blocks.filter(block => block.id !== blockId),
        pages: state.pages?.filter(page => page.id !== blockId)
      }));
    }
  },

  // Audio actions
  startRecording: async (pageId: string) => {
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        const recordingId: string = await invoke('start_recording', { pageId });

        set(state => ({
          audioState: {
            ...state.audioState,
            isRecording: true,
            recordingId,
            pageId,
            startTime: Date.now(),
          }
        }));
      } catch (error) {
        console.error("Error invoking start_recording:", error);
        set({ error: error as string });
        throw error; // Re-throw or handle
      }
    } else {
      console.warn('Tauri API (window.__TAURI__) not found. Skipping startRecording.');
      set(state => ({
        audioState: {
          ...state.audioState,
          isRecording: false, // Ensure recording state is not stuck
        },
        error: "Tauri API not available",
      }));
      // Do not throw error here to match original behavior of not throwing for UI updates
    }
  },

  stopRecording: async () => {
    const { audioState } = get();
    if (!audioState.recordingId) return;

    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        await invoke('stop_recording', { recordingId: audioState.recordingId });

        set(state => ({
          audioState: {
            ...state.audioState,
            isRecording: false,
            recordingId: undefined,
            pageId: undefined,
            startTime: undefined,
          }
        }));
      } catch (error) {
        console.error("Error invoking stop_recording:", error);
        set({ error: error as string });
        throw error; // Re-throw or handle
      }
    } else {
      console.warn('Tauri API (window.__TAURI__) not found. Skipping stopRecording.');
      set(state => ({
        audioState: {
          ...state.audioState,
          isRecording: false, // Ensure recording state is reset
          recordingId: undefined,
          pageId: undefined,
          startTime: undefined,
        },
        error: "Tauri API not available",
      }));
      // Do not throw error here to match original behavior of not throwing for UI updates
    }
  },

  loadAudioDevices: async () => {
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        const devices: AudioDevice[] = await invoke('get_audio_devices');

        set(state => ({
          audioState: {
            ...state.audioState,
            devices,
          }
        }));
      } catch (error) {
        console.error("Error invoking get_audio_devices:", error);
        set({ error: error as string });
        // Potentially re-throw or handle more gracefully depending on desired UX
        // For now, just setting the error state
      }
    } else {
      console.warn('Tauri API (window.__TAURI__) not found. Skipping loadAudioDevices.');
      // Optionally set devices to empty array or a default state
      set(state => ({
        audioState: {
          ...state.audioState,
          devices: [], // Ensure devices is not undefined
        }
      }));
    }
  },

  playAudioFromTimestamp: (audioTimestamp: AudioTimestamp) => {
    if (!audioTimestamp.recording) return;

    // Create audio element and play from timestamp
    const audio = new Audio(`file://${audioTimestamp.recording.file_path}`);
    audio.currentTime = audioTimestamp.timestamp_seconds;
    audio.play().catch(error => {
      console.error('Failed to play audio:', error);
    });
  },
}));

