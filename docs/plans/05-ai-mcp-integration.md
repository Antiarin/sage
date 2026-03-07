# Sage - AI / MCP / Agent Integration

## Philosophy

No existing language has first-class AI primitives. Sage is the first AI-native language. Built-in syntax for MCP servers, agents, and LLM calls. Community builds frameworks on top.

## MCP Server - Built-in Syntax

```
@mcp_server(name: "weather-tool", version: "1.0")
module WeatherServer {

    @tool(description: "Get weather for a city")
    fn get_weather(city: str) -> WeatherData {
        let data = weather_api.fetch(city)
        return WeatherData {
            temp: data.temp,
            condition: data.condition
        }
    }

    @resource("weather://{city}/current")
    fn weather_resource(city: str) -> str {
        return get_weather(city).to_json()
    }

    @prompt("weather-report")
    fn weather_prompt(city: str) -> str {
        let data = get_weather(city)
        return "Current weather in {city}: {data.temp}F, {data.condition}"
    }
}
```

## Agent - First-class Type

```
agent ResearchAgent {
    model: "claude-sonnet-4-20250514"
    tools: [WebSearch, FileRead, CodeRun]
    max_steps: 20

    system: "You are a research assistant."

    fn on_tool_call(call: ToolCall) -> ToolResult {
        log("Agent calling: {call.name}")
        return call.execute()
    }

    fn on_complete(result: AgentResult) {
        save_report(result.output)
    }
}

fn main() {
    let agent = ResearchAgent.new()
    let result = agent.run("Find papers on transformer architectures")
    println(result.output)
}
```

## LLM Calls - Standard Library

```
import std.ai

fn summarize(text: str) -> str {
    let response = ai.complete(
        model: "claude-sonnet-4-20250514",
        prompt: "Summarize: {text}",
        max_tokens: 500
    )
    return response.text
}

// Streaming
fn stream_response(prompt: str) {
    for chunk in ai.stream(model: "claude-sonnet-4-20250514", prompt: prompt) {
        print(chunk.text)
    }
}
```

## Tensor / ML - For Building LLMs

```
import std.tensor

fn linear_layer(input: Tensor[f32], weights: Tensor[f32], bias: Tensor[f32]) -> Tensor[f32] {
    return input.matmul(weights) + bias
}

fn attention(q: Tensor, k: Tensor, v: Tensor) -> Tensor {
    let scores = q.matmul(k.transpose()) / sqrt(k.shape[-1])
    let weights = softmax(scores)
    return weights.matmul(v)
}
```

## Standard Library Modules

| Module | Purpose |
|--------|---------|
| `std.ai` | LLM calls, completions, streaming |
| `std.mcp` | MCP protocol, server/client utilities |
| `std.agent` | Agent framework, tool management |
| `std.tensor` | Tensor math, SIMD, GPU compute |
| `std.http` | HTTP client/server |
| `std.json` | JSON parse/serialize |
| `std.fs` | File system operations |
| `std.net` | Networking, sockets |
| `std.crypto` | Cryptography primitives |
| `std.db` | Database drivers |
