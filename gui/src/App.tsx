import { useState } from "react";
import { Sidebar, type Feature } from "@/components/Sidebar";

function App() {
  const [activeFeature, setActiveFeature] = useState<Feature>("convert");
  const [showSettings, setShowSettings] = useState(false);

  return (
    <div className="flex h-screen bg-background text-foreground">
      <Sidebar
        active={activeFeature}
        onSelect={(feature) => {
          setActiveFeature(feature);
          setShowSettings(false);
        }}
        onSettingsClick={() => setShowSettings(true)}
      />
      <main className="flex-1 overflow-auto p-6">
        {showSettings ? (
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
            <p className="mt-1 text-muted-foreground">
              Application settings (TODO)
            </p>
          </div>
        ) : (
          <div>
            <h1 className="text-2xl font-bold capitalize tracking-tight">
              {activeFeature}
            </h1>
            <p className="mt-1 text-muted-foreground">
              Work area for {activeFeature}
            </p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
