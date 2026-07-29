# Bevy CLI Integration Goal

## Vision

Eliminate the artificial separation between "GUI" and "headless/CLI" modes. The Bevy app itself should be the headless interface.

## Core Insight

`ClickerCommand::Increment { count }` and Bevy systems like `increment_button` are already "commands" in essence. They're executable actions that don't inherently require a GUI or CLI - they're just functions that can be triggered from multiple contexts.

## Goal

Define a Bevy app once with systems that perform actions. These systems should be automatically usable from multiple contexts:

- **GUI**: mouse click triggers the system
- **CLI**: stdin command triggers the same system
- **Remote**: network command triggers the same system
- **Tests**: direct invocation

The "headless" mode isn't a separate implementation - it's the same Bevy app running without the GUI rendering layer.

## Implementation Approach

Use a decorator-like pattern (similar to Python decorators) to annotate systems and automatically expose them as CLI commands, eliminating manual CLI implementation.

## What to Avoid

Manually implementing `CliPlugin` with `read_stdin` that parses commands and sends `ClickerCommand`. Instead, the CLI should automatically derive from the available systems/commands.

## Core Principle

The Bevy app itself is the headless interface. The CLI is just a way to invoke the same systems that the GUI uses, but from stdin. There should be one set of systems, and multiple "triggers" (GUI events, CLI commands, network messages) that all invoke those systems.
