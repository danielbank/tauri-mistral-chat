import { ChatSection } from "./components/chat";
import "./App.css";

function App() {
  return (
    <main className="container mx-auto h-screen p-4">
      <div className="h-full max-w-4xl mx-auto">
        <header className="mb-6">
          <h1 className="text-3xl font-bold text-center">
            🦀 AI Chat with Multiple Models
          </h1>
          <p className="text-center text-muted-foreground mt-2">
            Powered by Tauri and mistral.rs
          </p>
        </header>

        <div className="h-[calc(100%-8rem)]">
          <ChatSection />
        </div>
      </div>
    </main>
  );
}

export default App;
