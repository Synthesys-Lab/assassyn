# Agents

This directory contains agent definitions for Claude Code. Agents are specialized AI assistants for complex tasks requiring isolated context and specific model configurations.

## Purpose

Agents provide isolated execution environments for complex, multi-step tasks. Each agent is defined as a markdown file with YAML frontmatter configuration.

## Organization

- Each agent is a `.md` file in the `agents/` directory
- Agent files include:
  - YAML frontmatter: Configuration (name, description, model, tools, skills)
  - Markdown content: Agent behavior specification and workflow

## Available Agents

- `code-review.md`: Comprehensive code review with enhanced quality standards using Opus model for long context analysis
