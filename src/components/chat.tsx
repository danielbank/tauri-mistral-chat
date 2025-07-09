import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChat } from "ai/react";
import {
  ChatSection as LlamaIndexChatSection,
  ChatMessages,
  ChatInput,
  useChatUI,
} from "@llamaindex/chat-ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

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

// Custom component to show file attachments
function FileAttachmentIndicator() {
  const { requestData } = useChatUI();

  // Check if there's a file in the request data
  const file = requestData?.file;

  if (!file) return null;

  return (
    <div className="flex flex-wrap gap-2 p-2 bg-gray-50 rounded-md mb-2">
      <div className="flex items-center gap-2 px-2 py-1 bg-white rounded border text-sm">
        <span className="text-blue-600">📎</span>
        <span className="font-medium">{file.name || "Unnamed file"}</span>
        <span className="text-gray-500 text-xs">
          {file.type || "Unknown type"}
        </span>
        <span className="text-gray-500 text-xs">
          ({Math.round(file.size / 1024)}KB)
        </span>
      </div>
    </div>
  );
}

// Model Selection Component
function ModelSelector({
  onModelSelect,
}: {
  onModelSelect: (modelId: string) => void;
}) {
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>("");
  const [isDiscoveringModels, setIsDiscoveringModels] = useState(true);

  // Auto-discover available local models on component mount
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
          onModelSelect(availableModel.id);
        }
      } catch (error) {
        console.error("Failed to discover models:", error);
      } finally {
        setIsDiscoveringModels(false);
      }
    }

    discoverModels();
  }, [onModelSelect]);

  // Update model selection when model changes
  useEffect(() => {
    if (selectedModel) {
      onModelSelect(selectedModel);
    }
  }, [selectedModel, onModelSelect]);

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

  const currentModel = availableModels.find((m) => m.id === selectedModel);

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-lg">🤖 AI Chat Configuration</CardTitle>
        <CardDescription>
          Select your AI model and configure chat settings
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {isDiscoveringModels ? (
          <p className="text-muted-foreground">
            🔍 Discovering available models...
          </p>
        ) : (
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Select AI Model:</label>
              <Select value={selectedModel} onValueChange={setSelectedModel}>
                <SelectTrigger>
                  <SelectValue placeholder="Select a model..." />
                </SelectTrigger>
                <SelectContent>
                  {availableModels.map((model) => (
                    <SelectItem
                      key={model.id}
                      value={model.id}
                      disabled={!model.is_available}
                    >
                      <div className="flex items-center gap-2">
                        <span>{getModelStatusIcon(model)}</span>
                        <span>{model.name}</span>
                        <Badge variant="secondary" className="text-xs">
                          {getModelTypeLabel(model)}
                        </Badge>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {selectedModel && currentModel && (
              <div className="text-sm text-muted-foreground space-y-1">
                <p>
                  <strong>📝 Description:</strong> {currentModel.description}
                </p>
                {currentModel.repo && (
                  <p>
                    <strong>🔗 Repository:</strong> {currentModel.repo}
                  </p>
                )}
                <div className="flex items-center gap-2">
                  <strong>📊 Type:</strong> {getModelTypeLabel(currentModel)}
                  {currentModel.is_vision && (
                    <Badge variant="outline" className="text-xs">
                      Vision Support
                    </Badge>
                  )}
                </div>
                {!currentModel.is_available && (
                  <p className="text-destructive">
                    ⚠️ This model is not available. Check your HF_TOKEN or
                    download the model locally.
                  </p>
                )}
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function ChatSection() {
  const [selectedModelId, setSelectedModelId] = useState<string>("");
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);

  // Get available models to check if selected model supports vision
  useEffect(() => {
    async function getModels() {
      try {
        const models = await invoke<ModelInfo[]>("discover_models");
        setAvailableModels(models);
      } catch (error) {
        console.error("Failed to get models:", error);
      }
    }
    getModels();
  }, []);

  // Create the chat handler using useChat hook
  const handler = useChat({
    api: "/api/chat", // This won't be used since we override append
  });

  console.log("=== CHAT HANDLER CREATED ===");
  console.log("Handler object:", handler);
  console.log("Original append function:", handler.append);
  console.log("Original handleSubmit function:", handler.handleSubmit);

  // Store the original functions
  const originalAppend = handler.append;
  const originalHandleSubmit = handler.handleSubmit;

  // Helper function for image conversion from Data URL
  function extractBase64FromDataUrl(dataUrl: string): string {
    // Data URL format: data:image/jpeg;base64,/9j/4AAQ...
    const base64Part = dataUrl.split(",")[1];
    return base64Part;
  }

  // Helper function to convert File to base64
  async function convertFileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        // Extract base64 part from data URL
        const base64 = result.split(",")[1];
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  // Helper function to check if file is an image
  function isImageFile(file: File): boolean {
    return file.type.startsWith("image/");
  }

  // Override the append function to use Tauri backend
  handler.append = async (message: any, options?: any) => {
    console.log("=== APPEND OVERRIDE CALLED ===");
    console.log("Full message object:", JSON.stringify(message, null, 2));
    console.log("Options:", JSON.stringify(options, null, 2));
    console.log("Message role:", message.role);
    console.log("Message content:", message.content);
    console.log("Options data:", options?.data);

    // Add user message first using original append
    const result = await originalAppend(message, options);

    // Only process if this is a user message
    if (message.role !== "user") {
      console.log("Not a user message, skipping processing");
      return result;
    }

    try {
      // Use the selected model or fallback
      const modelId = selectedModelId || "llama-3.2-3b-instruct"; // fallback model

      // Check if the selected model supports vision
      const selectedModel = availableModels.find((m) => m.id === modelId);
      const modelSupportsVision = selectedModel?.is_vision || false;

      console.log("Selected model:", selectedModel?.name || modelId);
      console.log("Model supports vision:", modelSupportsVision);

      // Process file from options.data
      let imageData: string | undefined = undefined;
      const file = options?.data?.file;

      console.log("Processing file:", file);

      if (file) {
        console.log("File details:", {
          name: file.name,
          type: file.type,
          size: file.size,
        });

        if (isImageFile(file)) {
          console.log("Found image file!");

          if (!modelSupportsVision) {
            console.log("Model doesn't support vision, showing error");
            await originalAppend({
              role: "assistant",
              content: `Error: The selected model "${
                selectedModel?.name || modelId
              }" does not support vision/image inputs. Please select a vision-capable model to analyze images.`,
            });
            return result;
          }

          // Convert File to base64
          try {
            imageData = await convertFileToBase64(file);
            console.log("Successfully converted file to base64");
            console.log("Base64 length:", imageData?.length);
            console.log(
              "Base64 preview (first 50 chars):",
              imageData?.substring(0, 50)
            );
          } catch (error) {
            console.error("Failed to convert file to base64:", error);
            await originalAppend({
              role: "assistant",
              content: `Error: Failed to process image file: ${error}`,
            });
            return result;
          }
        } else {
          console.log("Non-image file uploaded:", file.type);
          await originalAppend({
            role: "assistant",
            content: `Error: Only image files are supported for vision models. You uploaded: ${file.type}`,
          });
          return result;
        }
      }

      // Call Tauri backend
      console.log("Calling Tauri backend with:");
      console.log("- message:", message.content);
      console.log("- modelId:", modelId);
      console.log("- hasImage:", !!imageData);
      console.log("- imageDataLength:", imageData?.length || 0);

      const response = await invoke<string>("ai_chat", {
        message: message.content,
        modelId: modelId,
        imageData: imageData,
      });

      console.log("Received response from Tauri backend:", response);

      // Add AI response using original append
      await originalAppend({
        role: "assistant",
        content: response,
      });
    } catch (error) {
      console.error("Error in append function:", error);
      // Add error message using original append
      await originalAppend({
        role: "assistant",
        content: `Error: ${error}`,
      });
    }

    return result;
  };

  // Also try overriding handleSubmit
  handler.handleSubmit = async (event?: any, options?: any) => {
    console.log("=== HANDLE SUBMIT OVERRIDE CALLED ===");
    console.log("Event:", event);
    console.log("Options:", options);
    console.log("Handler input:", handler.input);
    console.log("Handler messages:", handler.messages);

    // Try to call original submit but with our processing
    return originalHandleSubmit(event, options);
  };

  console.log("=== OVERRIDES APPLIED ===");
  console.log("New append function:", handler.append);
  console.log("New handleSubmit function:", handler.handleSubmit);

  return (
    <div className="flex h-full flex-col gap-4">
      {/* Model Selection Header */}
      <ModelSelector onModelSelect={setSelectedModelId} />

      {/* Chat Interface using LlamaIndex Chat UI */}
      <Card className="flex-1 min-h-0">
        <CardContent className="p-4 h-full">
          <LlamaIndexChatSection
            handler={handler}
            className="h-full flex flex-col"
          >
            <ChatMessages className="flex-1 min-h-80" />
            <ChatInput>
              <FileAttachmentIndicator />
              <ChatInput.Form>
                <ChatInput.Field placeholder="Ask the AI anything..." />
                <ChatInput.Upload />
                <ChatInput.Submit />
              </ChatInput.Form>
            </ChatInput>
          </LlamaIndexChatSection>
        </CardContent>
      </Card>
    </div>
  );
}
