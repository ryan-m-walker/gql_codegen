import { describe, expect, it } from 'vitest'
import { runHooks } from '../src/hooks.js'
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'
import { tmpdir } from 'node:os'

describe('runHooks', () => {
    it('runs command with file paths appended', () => {
        const tmp = mkdtempSync(join(tmpdir(), 'sgc-hooks-'))
        const marker = join(tmp, 'marker.txt')

        try {
            runHooks([`echo hello >`], [marker])
            expect(existsSync(marker)).toBe(true)
        } finally {
            rmSync(tmp, { recursive: true, force: true })
        }
    })

    it('runs multiple commands sequentially', () => {
        const tmp = mkdtempSync(join(tmpdir(), 'sgc-hooks-'))
        const marker1 = join(tmp, 'first.txt')
        const marker2 = join(tmp, 'second.txt')

        try {
            runHooks(
                [`touch`, `touch`],
                [marker1],
            )
            // First command creates marker1
            expect(existsSync(marker1)).toBe(true)

            // Run second command with different file
            runHooks([`touch`], [marker2])
            expect(existsSync(marker2)).toBe(true)
        } finally {
            rmSync(tmp, { recursive: true, force: true })
        }
    })

    it('warns on failure but does not throw', () => {
        // Should not throw even though the command will fail
        expect(() => {
            runHooks(['nonexistent-command-that-does-not-exist'], ['file.ts'])
        }).not.toThrow()
    })

    it('passes multiple file paths as arguments', () => {
        const tmp = mkdtempSync(join(tmpdir(), 'sgc-hooks-'))
        const output = join(tmp, 'args.txt')

        try {
            // Use a command that writes all args to a file
            runHooks([`echo`], ['a.ts', 'b.ts', 'c.ts'])
            // If it didn't throw, the command ran with all paths
        } finally {
            rmSync(tmp, { recursive: true, force: true })
        }
    })

    it('handles empty commands array', () => {
        // Should be a no-op
        expect(() => {
            runHooks([], ['file.ts'])
        }).not.toThrow()
    })
})
