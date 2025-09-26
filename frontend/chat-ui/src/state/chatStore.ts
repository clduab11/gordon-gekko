import { create } from 'zustand';
import { ChatMessage, DiagnosticLog } from '../types';

interface ChatStore {
  messages: ChatMessage[];
  diagnostics: DiagnosticLog[];
  setMessages: (messages: ChatMessage[]) => void;
  appendMessage: (message: ChatMessage) => void;
  setDiagnostics: (diagnostics: DiagnosticLog[]) => void;
}

export const useChatStore = create<ChatStore>((set) => ({
  messages: [],
  diagnostics: [],
  setMessages: (messages) => set({ messages }),
  appendMessage: (message) =>
    set((state) => ({
      messages: [...state.messages, message]
    })),
  setDiagnostics: (diagnostics) => set({ diagnostics })
}));
