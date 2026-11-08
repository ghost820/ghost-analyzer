import { DockviewReact, type DockviewApi, type DockviewReadyEvent, type IDockviewPanelProps } from "dockview-react";
import { Header } from "./components/Header";
import { ScopePanel, SpectrumPanel } from "./panels/PlotPanel";
import { SourcePanel } from "./panels/SourcePanel";

const components = {
  scope: (_: IDockviewPanelProps) => <ScopePanel />,
  spectrum: (_: IDockviewPanelProps) => <SpectrumPanel />,
  source: (_: IDockviewPanelProps) => <SourcePanel />,
};

const theme = {
  name: "ghost",
  className: "dockview-theme-ghost",
  colorScheme: "dark" as const,
  gap: 6,
  dndPanelOverlay: "group" as const,
};

function buildLayout(api: DockviewApi): void {
  const scope = api.addPanel({ id: "scope", component: "scope", title: "OSCILLOSCOPE" });
  api.addPanel({
    id: "spectrum",
    component: "spectrum",
    title: "SPECTRUM ANALYZER",
    position: { referencePanel: scope, direction: "below" },
  });
  api.addPanel({
    id: "source",
    component: "source",
    title: "SIGNAL SOURCE",
    position: { referencePanel: scope, direction: "right" },
    initialWidth: 340,
  });
}

export function App() {
  const onReady = (event: DockviewReadyEvent) => buildLayout(event.api);
  return (
    <div className="app">
      <Header />
      <main className="workspace">
        <DockviewReact components={components} onReady={onReady} theme={theme} />
      </main>
    </div>
  );
}
