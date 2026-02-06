export interface GenerateOptions {
    /** JSON string of the config */
    configJson: string;
    /** Whether to skip caching */
    noCache?: boolean;
    /** Whether to enable timing output */
    timing?: boolean;
    /** Max diagnostics to show per error group (0 = unlimited, default 3) */
    maxDiagnostics?: number;
}
export interface GeneratedFile {
    path: string;
    content: string;
}
export interface GenerateResult {
    /** Whether generation was skipped (cache hit) */
    fresh: boolean;
    /** Generated files (only populated if not fresh) */
    files: GeneratedFile[];
    /** Warnings encountered during generation */
    warnings: string[];
}
export interface WriteError {
    path: string;
    message: string;
}
export interface WriteFilesResult {
    /** Paths that were written */
    written: string[];
    /** Paths skipped because content already matched */
    skipped: string[];
    /** Paths that failed to write */
    errors: WriteError[];
}
/**
 * Generate TypeScript types from GraphQL schema and operations
 */
export declare function generate(options: GenerateOptions): GenerateResult;
/**
 * Clear the cache directory
 * @param baseDir - Base directory containing .sgc cache
 * @returns Whether cache was cleared
 */
export declare function clearCache(baseDir: string): boolean;
/**
 * Write generated files to disk using parallel I/O.
 * Skips files whose content already matches (avoids unnecessary fs events).
 */
export declare function writeFiles(files: GeneratedFile[]): WriteFilesResult;
/**
 * Check if native binding is available
 */
export declare function isNativeAvailable(): boolean;
/**
 * Get the load error if native binding failed
 */
export declare function getLoadError(): Error | null;
//# sourceMappingURL=index.d.ts.map