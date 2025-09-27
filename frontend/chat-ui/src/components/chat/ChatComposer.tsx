import { FormEvent, useState } from 'react';
import { Loader2, Send, Upload } from 'lucide-react';

interface Props {
  disabled?: boolean;
  onSend: (prompt: string) => void;
}

const ChatComposer = ({ disabled, onSend }: Props) => {
  const [value, setValue] = useState('');

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!value.trim()) return;
    onSend(value.trim());
    setValue('');
  };

  return (
    <form onSubmit={handleSubmit} className="border-t border-border/60 p-4">
      <div className="rounded-xl border border-border/60 bg-panel px-4 py-3">
        <textarea
          className="h-24 w-full resize-none border-none bg-transparent text-sm text-white/90 outline-none"
          placeholder="Ask Gordon to orchestrate trades, research, or automation..."
          value={value}
          onChange={(event) => setValue(event.target.value)}
          disabled={disabled}
        />
        <div className="mt-3 flex items-center justify-between text-xs text-white/60">
          <label className="flex cursor-pointer items-center gap-2 rounded-full border border-border/80 px-3 py-2 hover:border-accent/70">
            <Upload className="h-4 w-4" />
            Attach (CSV, PDF, MD)
            <input type="file" className="hidden" multiple />
          </label>
          <button
            type="submit"
            className="flex items-center gap-2 rounded-full bg-accent px-4 py-2 font-semibold text-black disabled:cursor-not-allowed disabled:bg-white/40"
            disabled={disabled}
          >
            {disabled ? <Loader2 className="h-4 w-4 animate-spin" /> : <Send className="h-4 w-4" />} Send
          </button>
        </div>
      </div>
    </form>
  );
};

export default ChatComposer;
