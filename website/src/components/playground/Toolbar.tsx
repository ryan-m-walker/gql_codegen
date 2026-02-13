import type { GenerationResult } from './types'

interface ToolbarProps {
    isGenerating: boolean
    result: GenerationResult | null
    onShare: () => void
    shareMessage: string | null
}

export default function Toolbar({
    isGenerating,
    result,
    onShare,
    shareMessage,
}: ToolbarProps) {
    return (
        <div className="flex items-center justify-between px-4 py-2 border-b border-border-default bg-surface-raised">
            <div className="flex items-center gap-4">
                <h1 className="text-lg font-semibold">Playground</h1>
                {isGenerating ? (
                    <span className="text-xs text-text-muted flex items-center gap-2">
                        <svg
                            className="animate-spin h-3 w-3"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                        >
                            <circle
                                className="opacity-25"
                                cx="12"
                                cy="12"
                                r="10"
                                stroke="currentColor"
                                strokeWidth="4"
                            />
                            <path
                                className="opacity-75"
                                fill="currentColor"
                                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            />
                        </svg>
                        Generating...
                    </span>
                ) : result ? (
                    <span className="text-xs text-text-muted">
                        <span
                            className={
                                result.error
                                    ? 'text-red-400'
                                    : 'text-green-400'
                            }
                        >
                            {result.timeMs.toFixed(1)}ms
                        </span>
                    </span>
                ) : null}
            </div>
            <div className="flex items-center gap-3">
                <button
                    onClick={onShare}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-text-secondary hover:text-text-heading bg-surface-inset hover:opacity-80 rounded transition-colors"
                >
                    <svg
                        className="w-3.5 h-3.5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"
                        />
                    </svg>
                    {shareMessage || 'Share'}
                </button>
            </div>
        </div>
    )
}
