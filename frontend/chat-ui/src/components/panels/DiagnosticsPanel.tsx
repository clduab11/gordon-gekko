import { Activity, AlertTriangle, Info } from 'lucide-react';
import { DiagnosticLog } from '../../types';

interface Props {
  diagnostics: DiagnosticLog[];
}

const iconBySeverity: Record<DiagnosticLog['severity'], JSX.Element> = {
  info: <Info className="h-4 w-4" />,
  warning: <AlertTriangle className="h-4 w-4" />,
  critical: <Activity className="h-4 w-4" />
};

const colorBySeverity: Record<DiagnosticLog['severity'], string> = {
  info: 'border border-accent/40 bg-accent/10 text-accent',
  warning: 'border border-yellow-400/60 bg-yellow-400/10 text-yellow-200',
  critical: 'border border-red-500/60 bg-red-500/10 text-red-200'
};

const DiagnosticsPanel = ({ diagnostics }: Props) => {
  if (!diagnostics.length) {
    return null;
  }

  return (
    <section className="rounded-xl border border-border/60 bg-panel/80 p-4 text-sm">
      <header className="text-xs uppercase tracking-[0.35em] text-white/40">Diagnostics</header>
      <ul className="mt-3 space-y-3">
        {diagnostics.map((log) => (
          <li key={log.id} className={`rounded-lg px-3 py-2 text-xs ${colorBySeverity[log.severity]}`}>
            <div className="flex items-center gap-2 font-semibold uppercase tracking-[0.3em]">
              {iconBySeverity[log.severity]}
              <span>{log.label}</span>
            </div>
            <p className="mt-2 text-[13px] leading-relaxed text-white/80">{log.detail}</p>
          </li>
        ))}
      </ul>
    </section>
  );
};

export default DiagnosticsPanel;
