import { useMemo, useState } from 'react';
import { Loader2, Pause, Play, Rocket, Sparkles, SquareChartGantt } from 'lucide-react';

import { useChatController } from './hooks/useChatController';
import PersonaControls from './components/panels/PersonaControls';
import InsightsPanel from './components/panels/InsightsPanel';
import ChatConversation from './components/chat/ChatConversation';
import ChatComposer from './components/chat/ChatComposer';
import ActionDashboard from './components/panels/ActionDashboard';
import DiagnosticsPanel from './components/panels/DiagnosticsPanel';

function App() {
  const { messages, persona, diagnostics, isSending, isPersonaLoading, sendMessage, savePersona } =
    useChatController();
  const [duration, setDuration] = useState(4);

  const personaLabel = useMemo(() => `${persona.tone} · ${persona.style} · ${persona.mood}`, [persona]);

  return (
    <div className="min-h-screen bg-background text-white">
      <header className="flex items-center justify-between px-10 py-6 border-b border-border/60">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Talk to Gordon</h1>
          <p className="text-sm text-white/70">Institutional-grade agentic control for Ninja Gekko</p>
        </div>
        <div className="flex items-center gap-3 text-sm text-white/70">
          <Sparkles className="h-4 w-4 text-accent" />
          <span>Persona: {personaLabel}</span>
          {isPersonaLoading ? <Loader2 className="h-4 w-4 animate-spin text-accent" /> : null}
        </div>
      </header>

      <main className="grid grid-cols-[2.1fr_1.2fr] gap-6 px-10 py-8">
        <section className="flex flex-col gap-4">
          <div className="flex items-center justify-between rounded-xl border border-border/40 bg-panel/80 px-6 py-4">
            <div>
              <h2 className="text-lg font-semibold">Live Orchestration</h2>
              <p className="text-sm text-white/60">Control trading automations, research swarms, and MPC flows.</p>
            </div>
            <div className="flex items-center gap-3 text-sm">
              <button className="flex items-center gap-2 rounded-lg border border-accentSoft/60 px-3 py-2 font-medium text-accent">
                <Play className="h-4 w-4" /> Resume
              </button>
              <button className="flex items-center gap-2 rounded-lg border border-border px-3 py-2 font-medium text-white/80 hover:border-accent">
                <Pause className="h-4 w-4" /> Pause {duration}h
              </button>
              <input
                type="range"
                min={1}
                max={24}
                value={duration}
                onChange={(event) => setDuration(Number(event.target.value))}
                className="h-1 w-32 accent-accent"
                aria-label="Pause duration"
              />
            </div>
          </div>

          <div className="flex h-[65vh] gap-4">
            <div className="flex w-full flex-col rounded-xl border border-border/40 bg-panel/80">
              <div className="flex items-center justify-between border-b border-border/60 px-6 py-4">
                <div>
                  <h2 className="text-lg font-semibold">Conversation</h2>
                  <p className="text-xs uppercase tracking-[0.28em] text-white/40">Memory · Citations · Control</p>
                </div>
                <span className="flex items-center gap-2 rounded-full bg-accentSoft/20 px-3 py-1 text-xs text-accent">
                  <SquareChartGantt className="h-4 w-4" /> Autonomous Mode
                </span>
              </div>
              <ChatConversation messages={messages} />
              <ChatComposer disabled={isSending} onSend={sendMessage} />
            </div>

            <div className="flex w-[28rem] flex-col gap-4">
              <PersonaControls persona={persona} onSave={savePersona} isLoading={isPersonaLoading} />
              <DiagnosticsPanel diagnostics={diagnostics} />
              <ActionDashboard />
            </div>
          </div>
        </section>

        <aside className="flex flex-col gap-4">
          <InsightsPanel />
        </aside>
      </main>

      <footer className="flex items-center justify-between px-10 py-4 text-xs text-white/40">
        <span>© {new Date().getFullYear()} Ninja Gekko · Agentic Trading Intelligence</span>
        <span className="flex items-center gap-2">
          <Rocket className="h-4 w-4" /> MCP Mesh Active
        </span>
      </footer>
    </div>
  );
}

export default App;
