import fs from 'node:fs/promises'

/**
 * Check if a path exists.
 * fs.exists is deprecated so this is just a convenience wrapper over fs.access.
 */
export async function exists(p: string) {
    try {
        await fs.access(p)
        return true
    } catch {
        return false
    }
}
