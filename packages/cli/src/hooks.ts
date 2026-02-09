import { execSync } from 'node:child_process'

/**
 * Run a hook command with file paths as arguments.
 * Logs warnings on failure — does not throw.
 */
function runCommand(command: string, filePaths: string[]): void {
    try {
        execSync(`${command} ${filePaths.join(' ')}`, { stdio: 'inherit' })
    } catch {
        process.stderr.write(`[sgc] Hook "${command}" failed\n`)
    }
}

/**
 * Run an array of hook commands sequentially.
 */
export function runHooks(commands: string[], filePaths: string[]): void {
    for (const cmd of commands) {
        runCommand(cmd, filePaths)
    }
}
