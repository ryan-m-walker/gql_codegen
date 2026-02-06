import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { loadSchema, resolveSchemas } from '../src/schema.js'

const fixture = (name: string) =>
    path.resolve(import.meta.dirname, '__fixtures__', name)

describe('loadSchema', () => {
    it('returns null for .graphql files', async () => {
        const result = await loadSchema('schema.graphql')
        expect(result).toBeNull()
    })

    it('returns null for glob patterns', async () => {
        const result = await loadSchema('src/**/*.ts')
        expect(result).toBeNull()
    })

    it('returns null for non-existent files', async () => {
        const result = await loadSchema('/tmp/does-not-exist.ts')
        expect(result).toBeNull()
    })

    it('resolves default export of GraphQLSchema', async () => {
        const result = await loadSchema(fixture('schema-default-export.ts'))
        expect(result).not.toBeNull()
        expect(result!.sdl).toContain('type Query')
        expect(result!.sdl).toContain('hello')
    })

    it('resolves named "schema" export', async () => {
        const result = await loadSchema(fixture('schema-named-export.ts'))
        expect(result).not.toBeNull()
        expect(result!.sdl).toContain('type Query')
        expect(result!.sdl).toContain('world')
    })

    it('resolves string SDL export', async () => {
        const result = await loadSchema(fixture('schema-string-export.ts'))
        expect(result).not.toBeNull()
        expect(result!.sdl).toContain('greeting')
    })

    it('extracts codegenScalarType extensions', async () => {
        const result = await loadSchema(fixture('schema-with-scalars.ts'))
        expect(result).not.toBeNull()
        expect(result!.scalars).toEqual({
            DateTime: 'Date | string',
            JSON: { input: 'Record<string, unknown>', output: 'unknown' },
        })
    })

    it('throws for modules with no valid schema export', async () => {
        await expect(
            loadSchema(fixture('schema-invalid.ts')),
        ).rejects.toThrow('does not export a valid schema')
    })
})

describe('resolveSchemas', () => {
    it('passes .graphql paths through to schemaPaths', async () => {
        const result = await resolveSchemas([
            'schema.graphql',
            'src/**/*.graphql',
        ])
        expect(result.schemaPaths).toEqual([
            'schema.graphql',
            'src/**/*.graphql',
        ])
        expect(result.schemaContent).toEqual([])
        expect(result.scalars).toEqual({})
    })

    it('resolves .ts schemas and splits from static paths', async () => {
        const result = await resolveSchemas([
            'schema.graphql',
            fixture('schema-default-export.ts'),
        ])
        expect(result.schemaPaths).toEqual(['schema.graphql'])
        expect(result.schemaContent).toHaveLength(1)
        expect(result.schemaContent[0]).toContain('type Query')
    })

    it('merges scalar extensions from multiple schemas', async () => {
        const result = await resolveSchemas([
            fixture('schema-with-scalars.ts'),
        ])
        expect(result.scalars).toHaveProperty('DateTime')
        expect(result.scalars).toHaveProperty('JSON')
    })
})
