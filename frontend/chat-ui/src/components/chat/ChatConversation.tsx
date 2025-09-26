import { useMemo } from 'react';
import clsx from 'clsx';
import { ChatMessage } from '../../types';

interface Props {
  messages: ChatMessage[];
}

const roleStyles: Record<ChatMessage['role'], string> = {
  user: 'bg-white/5 border border-white/10',
  assistant: 'bg-accentSoft/10 border border-accentSoft/30',
  system: 'bg-panel border border-border/60'
};

function formatTimestamp(value: string) {
  return new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

const ChatConversation = ({ messages }: Props) => {
  const rendered = useMemo(() => messages.slice(-80), [messages]);
  return (
    <div className="flex-1 overflow-y-auto px-6 py-6 space-y-4">
      {rendered.map((message) => (
        <article key={message.id} className={clsx('rounded-xl p-4 shadow-inner', roleStyles[message.role])}>
          <header className="flex items-center justify-between text-xs uppercase tracking-[0.3em] text-white/40">
            <span>{message.role === 'assistant' ? 'Gordon' : message.role}</span>
            <time>{formatTimestamp(message.timestamp)}</time>
          </header>
          <p className="mt-3 text-sm leading-relaxed text-white/90 whitespace-pre-line">{message.content}</p>
          {message.citations?.length ? (
            <ul className="mt-3 flex flex-wrap gap-2 text-[11px] text-white/60">
              {message.citations.map((citation, idx) => (
                <li key={idx} className="rounded-full border border-accent/40 px-3 py-1">
                  {citation.type === 'external' ? (
                    <a href={citation.url} target="_blank" rel="noreferrer" className="hover:text-accent">
                      {citation.title}
                    </a>
                  ) : (
                    <span>{citation.source}: {citation.detail}</span>
                  )}
                </li>
              ))}
            </ul>
          ) : null}
        </article>
      ))}
    </div>
  );
};

export default ChatConversation;
