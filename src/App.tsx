import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Model metadata interface matching the Rust backend
interface ModelInfo {
  id: string;
  name: string;
  description: string;
  model_type: string;
  size_estimate?: string;
  is_available: boolean;
  repo?: string;
  files: string[];
  is_vision: boolean;
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [aiMessage, setAiMessage] = useState("");
  const [aiResponse, setAiResponse] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>("");
  const [isDiscoveringModels, setIsDiscoveringModels] = useState(true);
  const [selectedImage, setSelectedImage] = useState<File | null>(null);
  const [imagePreview, setImagePreview] = useState<string | null>(null);

  // Auto-discover available local models on app startup
  useEffect(() => {
    async function discoverModels() {
      try {
        setIsDiscoveringModels(true);
        const models = await invoke<ModelInfo[]>("discover_models");
        setAvailableModels(models);

        // Select the first available model as default
        const availableModel = models.find((m) => m.is_available);
        if (availableModel) {
          setSelectedModel(availableModel.id);
        }
      } catch (error) {
        console.error("Failed to discover models:", error);
      } finally {
        setIsDiscoveringModels(false);
      }
    }

    discoverModels();
  }, []);

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  // Handle image selection for vision models
  function handleImageSelect(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedImage(file);

      // Create preview for UI
      const reader = new FileReader();
      reader.onload = (e) => {
        setImagePreview(e.target?.result as string);
      };
      reader.readAsDataURL(file);
    }
  }

  function clearImage() {
    setSelectedImage(null);
    setImagePreview(null);
  }

  // Convert image to base64 for Rust backend
  function convertImageToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        // Remove the data URL prefix to get just the base64 data
        const base64 = result.split(",")[1];
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  // Main chat function - sends message and optional image to AI model
  async function sendAiMessage() {
    if (!aiMessage.trim() || !selectedModel) return;

    // Check if vision model requires an image
    const currentModel = availableModels.find((m) => m.id === selectedModel);
    if (currentModel?.is_vision && !selectedImage) {
      setAiResponse("Vision models require an image to be uploaded.");
      return;
    }

    setIsLoading(true);
    setAiResponse("Thinking...");

    try {
      let imageData: string | undefined;
      if (selectedImage && currentModel?.is_vision) {
        imageData = await convertImageToBase64(selectedImage);
      }

      const response = await invoke("ai_chat", {
        message: aiMessage,
        modelId: selectedModel,
        imageData: imageData,
      });
      setAiResponse(response as string);
    } catch (error) {
      setAiResponse(`Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  // UI helper functions for model display
  const getModelStatusIcon = (model: ModelInfo) => {
    if (!model.is_available) return "⚠️";
    if (model.is_vision) return "👁️";
    if (model.model_type.startsWith("local-")) return "💾";
    return "🌐";
  };

  const getModelTypeLabel = (model: ModelInfo) => {
    if (model.model_type === "remote-gguf") return "Remote";
    if (model.model_type === "remote-vision") return "Remote Vision";
    if (model.model_type === "local-gguf") return "Local GGUF";
    if (model.model_type === "local-matformer") return "Local MatFormer";
    if (model.model_type === "local-matformer-vision") return "Local Vision";
    return model.model_type;
  };

  return (
    <main className="container">
      <h1>🦀 AI Chat with Multiple Models</h1>

      {/* Model Selection Section */}
      <div
        style={{
          marginBottom: "2rem",
          padding: "1rem",
          backgroundColor: "#1a1a1a",
          borderRadius: "8px",
          border: "1px solid #333",
        }}
      >
        <h3>🤖 Model Selection</h3>

        {isDiscoveringModels ? (
          <p>🔍 Discovering available models...</p>
        ) : (
          <>
            <div style={{ marginBottom: "1rem" }}>
              <label
                htmlFor="model-select"
                style={{ display: "block", marginBottom: "0.5rem" }}
              >
                Select AI Model:
              </label>
              <select
                id="model-select"
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                disabled={isLoading}
                style={{
                  width: "100%",
                  padding: "0.5rem",
                  backgroundColor: "#333",
                  color: "white",
                  border: "1px solid #555",
                  borderRadius: "4px",
                }}
              >
                <option value="">Select a model...</option>
                {availableModels.map((model) => (
                  <option
                    key={model.id}
                    value={model.id}
                    disabled={!model.is_available}
                  >
                    {getModelStatusIcon(model)} {model.name} (
                    {getModelTypeLabel(model)})
                    {model.size_estimate && ` - ${model.size_estimate}`}
                  </option>
                ))}
              </select>
            </div>

            {selectedModel && (
              <div style={{ fontSize: "0.9rem", color: "#ccc" }}>
                {(() => {
                  const model = availableModels.find(
                    (m) => m.id === selectedModel
                  );
                  return model ? (
                    <div>
                      <p>
                        <strong>📝 Description:</strong> {model.description}
                      </p>
                      {model.repo && (
                        <p>
                          <strong>🔗 Repository:</strong> {model.repo}
                        </p>
                      )}
                      <p>
                        <strong>📊 Type:</strong> {getModelTypeLabel(model)}
                      </p>
                      {model.size_estimate && (
                        <p>
                          <strong>💾 Size:</strong> {model.size_estimate}
                        </p>
                      )}
                      {!model.is_available && (
                        <p style={{ color: "#ff6b6b" }}>
                          ⚠️ This model is not available. Check your HF_TOKEN or
                          download the model locally.
                        </p>
                      )}
                    </div>
                  ) : null;
                })()}
              </div>
            )}
          </>
        )}
      </div>

      <p>Try asking: "Hello! Can you write a simple function in Rust?"</p>

      {/* Image Upload Section for Vision Models */}
      {availableModels.find((m) => m.id === selectedModel)?.is_vision && (
        <div
          style={{
            marginBottom: "1rem",
            padding: "1rem",
            backgroundColor: "#1a1a2e",
            borderRadius: "8px",
            border: "1px solid #333",
          }}
        >
          <h4>📷 Image Upload</h4>
          <input
            type="file"
            accept="image/*"
            onChange={handleImageSelect}
            disabled={isLoading}
            style={{
              marginBottom: "0.5rem",
              width: "100%",
              padding: "0.5rem",
              backgroundColor: "#333",
              color: "white",
              border: "1px solid #555",
              borderRadius: "4px",
            }}
          />

          {imagePreview && (
            <div style={{ marginTop: "0.5rem" }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  marginBottom: "0.5rem",
                }}
              >
                <span style={{ fontSize: "0.9rem", color: "#ccc" }}>
                  Preview:
                </span>
                <button
                  type="button"
                  onClick={clearImage}
                  disabled={isLoading}
                  style={{
                    padding: "0.25rem 0.5rem",
                    fontSize: "0.8rem",
                    backgroundColor: "#ff4444",
                    color: "white",
                    border: "none",
                    borderRadius: "4px",
                    cursor: "pointer",
                  }}
                >
                  Clear
                </button>
              </div>
              <img
                src={imagePreview}
                alt="Preview"
                style={{
                  maxWidth: "200px",
                  maxHeight: "200px",
                  borderRadius: "4px",
                  border: "1px solid #555",
                }}
              />
            </div>
          )}
        </div>
      )}

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          sendAiMessage();
        }}
      >
        <input
          id="ai-input"
          value={aiMessage}
          onChange={(e) => setAiMessage(e.currentTarget.value)}
          placeholder="Ask the AI anything..."
          disabled={isLoading || !selectedModel || isDiscoveringModels}
        />
        <button
          type="submit"
          disabled={
            isLoading ||
            !aiMessage.trim() ||
            !selectedModel ||
            isDiscoveringModels
          }
        >
          {isLoading ? "Thinking..." : "Send"}
        </button>
      </form>

      {!selectedModel && !isDiscoveringModels && (
        <div
          style={{
            marginTop: "1rem",
            padding: "1rem",
            backgroundColor: "#4a1a1a",
            borderRadius: "8px",
            border: "1px solid #665",
          }}
        >
          ⚠️ Please select a model to start chatting
        </div>
      )}

      {aiResponse && (
        <div
          style={{
            marginTop: "1rem",
            padding: "1rem",
            backgroundColor: "#1a1a1a",
            borderRadius: "8px",
            border: "1px solid #333",
            whiteSpace: "pre-wrap",
            textAlign: "left",
          }}
        >
          <strong>
            🤖 AI Response{" "}
            {selectedModel &&
              `(${availableModels.find((m) => m.id === selectedModel)?.name})`}
            :
          </strong>
          <br />
          {aiResponse}
        </div>
      )}

      {/* Demo Information: Show discovered models */}
      {availableModels.length > 0 && (
        <details
          style={{ marginTop: "2rem", fontSize: "0.8rem", color: "#999" }}
        >
          <summary>📋 Available Models ({availableModels.length})</summary>
          <div style={{ marginTop: "1rem" }}>
            {availableModels.map((model) => (
              <div
                key={model.id}
                style={{
                  marginBottom: "1rem",
                  padding: "0.5rem",
                  backgroundColor: model.is_available ? "#1a2a1a" : "#2a1a1a",
                  borderRadius: "4px",
                  border: `1px solid ${model.is_available ? "#333" : "#665"}`,
                }}
              >
                <div>
                  {getModelStatusIcon(model)} <strong>{model.name}</strong>
                  <span style={{ marginLeft: "0.5rem", color: "#ccc" }}>
                    ({getModelTypeLabel(model)})
                  </span>
                </div>
                <div
                  style={{
                    fontSize: "0.7rem",
                    color: "#aaa",
                    marginTop: "0.25rem",
                  }}
                >
                  {model.description}
                </div>
                {model.size_estimate && (
                  <div style={{ fontSize: "0.7rem", color: "#888" }}>
                    Size: {model.size_estimate}
                  </div>
                )}
              </div>
            ))}
          </div>
        </details>
      )}
    </main>
  );
}

export default App;
