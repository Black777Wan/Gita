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
    // @ts-expect-error __TAURI__ is injected by Tauri
    if (window.__TAURI__) {
      try {
        const blocks: Block[] = await invoke('get_daily_note', { date });
        const currentPage = blocks.find(block => block.is_page);
        set({
          blocks,
          currentPage,
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
      console.warn('Tauri API (window.__TAURI__) not found. Skipping loadDailyNote.');
      set({ 
        isLoading: false,
        blocks: [],
        currentPage: undefined,
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
          set({
            blocks,
            currentPage: page,
            isLoading: false
          });
        } else {
          // Create new page
          // Ensure createBlock itself is guarded or this will fail if __TAURI__ is not present
          const newPage = await get().createBlock({
            content: undefined,
            parent_id: undefined,
            order: 0,
            is_page: true,
            page_title: title,
          });
          // If createBlock did nothing due to lack of Tauri, newPage might be undefined or represent an error.
          // This needs careful handling based on createBlock's guarded implementation.
          if (newPage) { // Assuming createBlock returns something meaningful or null/undefined
            set({
              blocks: [newPage],
              currentPage: newPage,
              isLoading: false
            });
          } else {
            // If newPage couldn't be created (e.g., Tauri not available), set appropriate state
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
      console.warn('Tauri API (window.__TAURI__) not found. Skipping loadPage.');
      set({ 
        isLoading: false,
        blocks: [],
        currentPage: undefined,
      });
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
        throw error; // Re-throw or handle as per desired UX
      }
    } else {
      console.warn('Tauri API (window.__TAURI__) not found. Skipping createBlock.');
      // Decide what to return or how to signal failure.
      // Throwing an error might be appropriate if the caller expects a Block.
      // Or return a specific value like null or undefined and let caller handle.
      // For now, logging and throwing an error.
      set({ error: "Tauri API not available" });
      throw new Error("Tauri API not available, cannot create block.");
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
        throw error; // Re-throw or handle
      }
    } else {
      console.warn('Tauri API (window.__TAURI__) not found. Skipping updateBlockContent.');
      set({ error: "Tauri API not available" });
      // Potentially throw an error or manage state to indicate failure
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
        throw error; // Re-throw or handle
      }
    } else {
      console.warn('Tauri API (window.__TAURI__) not found. Skipping deleteBlock.');
      set({ error: "Tauri API not available" });
      // Potentially throw an error or manage state to indicate failure
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

