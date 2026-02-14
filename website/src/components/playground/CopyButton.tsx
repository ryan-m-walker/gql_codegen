import { useState, useRef, useCallback } from 'react'

interface CopyButtonProps {
    getText: () => string
}

export default function CopyButton({ getText }: CopyButtonProps) {
    const [copied, setCopied] = useState(false)
    const timerRef = useRef<ReturnType<typeof setTimeout>>()

    const handleCopy = useCallback(() => {
        navigator.clipboard.writeText(getText())
        setCopied(true)
        clearTimeout(timerRef.current)
        timerRef.current = setTimeout(() => setCopied(false), 1500)
    }, [getText])

    return (
        <button
            type="button"
            onClick={handleCopy}
            className="text-xs text-text-faint hover:text-text-heading hover:bg-surface-raised px-2 py-1 rounded transition-colors flex items-center gap-1"
        >
            {copied ? (
                <>
                    <svg className="w-3 h-3 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    Copied!
                </>
            ) : (
                'Copy'
            )}
        </button>
    )
}
