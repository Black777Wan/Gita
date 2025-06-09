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
    try {
      const blocks: Block[] = await invoke('get_daily_note', { date });
      const currentPage = blocks.find(block => block.is_page);
      set({ 
        blocks, 
        currentPage,
        isLoading: false 
      });
    } catch (error) {
      set({ 
        error: error as string, 
        isLoading: false 
      });
    }
  },

  loadPage: async (title: string) => {
    set({ isLoading: true, error: undefined });
    try {
      const page: Block | null = await invoke('get_page_by_title', { title });
      if (page) {
        const children: Block[] = await invoke('get_block_children', { parentId: page.id });
        const blocks = [page, ...children];
        set({ 
          blocks, 
          currentPage: page,
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
        set({ 
          blocks: [newPage], 
          currentPage: newPage,
          isLoading: false 
        });
      }
    } catch (error) {
      set({ 
        error: error as string, 
        isLoading: false 
      });
    }
  },

  createBlock: async (blockData: CreateBlockRequest, audioMeta?: AudioMeta) => {
    try {
      const newBlock: Block = await invoke('create_block', { blockData, audioMeta });
      
      set(state => ({
        blocks: [...state.blocks, newBlock]
      }));
      
      return newBlock;
    } catch (error) {
      set({ error: error as string });
      throw error;
    }
  },

  updateBlockContent: async (blockId: string, content: string) => {
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
      set({ error: error as string });
      throw error;
    }
  },

  deleteBlock: async (blockId: string) => {
    try {
      await invoke('delete_block', { blockId });
      
      set(state => ({
        blocks: state.blocks.filter(block => block.id !== blockId)
      }));
    } catch (error) {
      set({ error: error as string });
      throw error;
    }
  },

  // Audio actions
  startRecording: async (pageId: string) => {
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
      set({ error: error as string });
      throw error;
    }
  },

  stopRecording: async () => {
    const { audioState } = get();
    if (!audioState.recordingId) return;

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
      set({ error: error as string });
      throw error;
    }
  },

  loadAudioDevices: async () => {
    try {
      const devices: AudioDevice[] = await invoke('get_audio_devices');
      
      set(state => ({
        audioState: {
          ...state.audioState,
          devices,
        }
      }));
    } catch (error) {
      set({ error: error as string });
      throw error;
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

