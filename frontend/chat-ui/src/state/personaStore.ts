import { create } from 'zustand';
import { PersonaSettings } from '../types';

interface PersonaStore {
  persona: PersonaSettings;
  setPersona: (persona: PersonaSettings) => void;
}

const defaultPersona: PersonaSettings = {
  tone: 'balanced',
  style: 'analytical',
  mood: 'direct'
};

export const usePersonaStore = create<PersonaStore>((set) => ({
  persona: defaultPersona,
  setPersona: (persona) => set({ persona })
}));
