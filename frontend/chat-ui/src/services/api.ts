import {
  AccountSnapshot,
  ChatMessage,
  ChatResponse,
  NewsHeadline,
  PauseTradingRequest,
  PauseTradingResponse,
  PersonaSettings,
  ResearchRequest,
  ResearchResponse,
  SwarmRequest,
  SwarmResponse
} from '../types';

const JSON_HEADERS = {
  'Content-Type': 'application/json'
};

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || 'Request failed');
  }
  return (await res.json()) as T;
}

export async function fetchChatHistory(): Promise<ChatMessage[]> {
  const res = await fetch('/api/chat/history');
  return handleResponse<ChatMessage[]>(res);
}

export async function sendChatMessage(prompt: string): Promise<ChatResponse> {
  const res = await fetch('/api/chat/message', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify({ prompt })
  });
  return handleResponse<ChatResponse>(res);
}

export async function fetchPersona(): Promise<PersonaSettings> {
  const res = await fetch('/api/chat/persona');
  return handleResponse<PersonaSettings>(res);
}

export async function updatePersona(persona: PersonaSettings): Promise<PersonaSettings> {
  const res = await fetch('/api/chat/persona', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(persona)
  });
  return handleResponse<PersonaSettings>(res);
}

export async function pauseTrading(payload: PauseTradingRequest): Promise<PauseTradingResponse> {
  const res = await fetch('/api/trading/pause', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(payload)
  });
  return handleResponse<PauseTradingResponse>(res);
}

export async function fetchAccountSnapshot(): Promise<AccountSnapshot> {
  const res = await fetch('/api/accounts/snapshot');
  return handleResponse<AccountSnapshot>(res);
}

export async function fetchNews(): Promise<NewsHeadline[]> {
  const res = await fetch('/api/news/headlines');
  return handleResponse<NewsHeadline[]>(res);
}

export async function requestResearch(payload: ResearchRequest): Promise<ResearchResponse> {
  const res = await fetch('/api/research/sonar', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(payload)
  });
  return handleResponse<ResearchResponse>(res);
}

export async function summonSwarm(payload: SwarmRequest): Promise<SwarmResponse> {
  const res = await fetch('/api/agents/swarm', {
    method: 'POST',
    headers: JSON_HEADERS,
    body: JSON.stringify(payload)
  });
  return handleResponse<SwarmResponse>(res);
}
