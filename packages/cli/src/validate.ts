import type { CodegenConfig } from "./types.js";

/**
 * Validate that an unknown config value has the fields Node needs to process.
 * Only checks fields used by the Node wrapper (schema, outputs).
 * Rust core handles deep validation of generator configs and documents.
 */
export function validateConfig(value: unknown): CodegenConfig {
    if (typeof value !== "object" || value === null) {
        throw new Error(
            "Invalid config: expected an object.\n" +
                "Make sure your config file exports a valid configuration.",
        );
    }

    const config = value as Record<string, unknown>;

    if (!("schema" in config)) {
        throw new Error(
            'Invalid config: missing "schema" field.\n' +
                'Add a schema path, e.g.: schema: "./schema.graphql"',
        );
    }

    if (typeof config.schema !== "string" && !Array.isArray(config.schema)) {
        throw new Error(
            'Invalid config: "schema" must be a string or array of strings.',
        );
    }

    if (
        Array.isArray(config.schema) &&
        !config.schema.every((s): s is string => typeof s === "string")
    ) {
        throw new Error(
            'Invalid config: "schema" array must only contain strings.',
        );
    }

    if (
        !("outputs" in config) ||
        typeof config.outputs !== "object" ||
        config.outputs === null
    ) {
        throw new Error(
            'Invalid config: missing "outputs" field.\n' +
                "Add output targets, e.g.:\n" +
                '  outputs: { "./src/types.ts": { generators: ["schema-types"] } }',
        );
    }

    // Structural validation done — Rust core validates the rest
    return value as CodegenConfig;
}
