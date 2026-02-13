import { useRef, useEffect } from 'react'

// --- Syntax-highlighted code segments ---

type Segment = { text: string; className: string }

const GQL_LINES: Segment[][] = [
    [
        { text: 'type', className: 'text-code-keyword' },
        { text: ' User ', className: 'text-code-value' },
        { text: '{', className: 'text-text-secondary' },
    ],
    [
        { text: '  id', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: 'ID!', className: 'text-code-value' },
    ],
    [
        { text: '  name', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: 'String!', className: 'text-code-value' },
    ],
    [
        { text: '  email', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: 'String', className: 'text-code-value' },
    ],
    [
        { text: '  posts', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: '[Post!]!', className: 'text-code-value' },
    ],
    [{ text: '}', className: 'text-text-secondary' }],
]

const TS_LINES: Segment[][] = [
    [
        { text: 'export', className: 'text-code-keyword' },
        { text: ' ', className: 'text-text-primary' },
        { text: 'interface', className: 'text-code-keyword' },
        { text: ' User ', className: 'text-code-value' },
        { text: '{', className: 'text-text-secondary' },
    ],
    [
        { text: '  ', className: 'text-text-primary' },
        { text: 'readonly', className: 'text-code-keyword' },
        { text: ' id', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: 'string', className: 'text-code-value' },
        { text: ';', className: 'text-text-secondary' },
    ],
    [
        { text: '  ', className: 'text-text-primary' },
        { text: 'readonly', className: 'text-code-keyword' },
        { text: ' name', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: 'string', className: 'text-code-value' },
        { text: ';', className: 'text-text-secondary' },
    ],
    [
        { text: '  ', className: 'text-text-primary' },
        { text: 'readonly', className: 'text-code-keyword' },
        { text: ' email', className: 'text-text-primary' },
        { text: '?: ', className: 'text-text-secondary' },
        { text: 'string', className: 'text-code-value' },
        { text: ' | ', className: 'text-text-secondary' },
        { text: 'null', className: 'text-code-keyword' },
        { text: ';', className: 'text-text-secondary' },
    ],
    [
        { text: '  ', className: 'text-text-primary' },
        { text: 'readonly', className: 'text-code-keyword' },
        { text: ' posts', className: 'text-text-primary' },
        { text: ': ', className: 'text-text-secondary' },
        { text: 'ReadonlyArray', className: 'text-code-value' },
        { text: '<Post>', className: 'text-text-secondary' },
        { text: ';', className: 'text-text-secondary' },
    ],
    [{ text: '}', className: 'text-text-secondary' }],
]

// --- Scramble character selection ---

const chars = [
    ...Array.from({ length: 10 }, () => '█'),
    ...Array.from({ length: 10 }, () => '░'),
    ...Array.from({ length: 10 }, () => '▒'),
    ...Array.from({ length: 10 }, () => '▓'),
    // ...Array.from({ length: 10 }, () => '┃'),
    // ...Array.from({ length: 10 }, () => '╋'),
    ...Array.from({ length: 10 }, () => '─'),
    ...Array.from({ length: 10 }, () => '<'),
    ...Array.from({ length: 10 }, () => '>'),
    ...Array.from({ length: 10 }, () => '{'),
    ...Array.from({ length: 10 }, () => '}'),
    ...Array.from({ length: 10 }, () => ';'),
    ...Array.from({ length: 10 }, () => ':'),
    ...Array.from({ length: 10 }, () => '='),
    // ...Array.from({ length: 10 }, () => '→'),
    // ...Array.from({ length: 10 }, () => '←'),
    ...Array.from({ length: 10 }, () => '∷'),
    ...Array.from({ length: 10 }, () => '?'),
    ...Array.from({ length: 1 }, () => '🦀'),
]

function getScrambleChar(): string {
    const index = Math.floor(Math.random() * chars.length)
    return chars[index]
}

// --- Helpers ---

function segmentsToString(segments: Segment[]): string {
    return segments.map((s) => s.text).join('')
}

/** Find the CSS class for the character at `index` within a line's segments */
function charClassAt(segments: Segment[], index: number): string {
    let offset = 0
    for (const seg of segments) {
        if (index < offset + seg.text.length) return seg.className
        offset += seg.text.length
    }
    return 'text-text-primary'
}

// --- Animation timing constants ---

const INITIAL_DELAY = 1500
const LINE_STAGGER = 250
const SCRAMBLE_DURATION = 700
const RESOLVE_DURATION = 500
const PAUSE_DURATION = 3500
const SCRAMBLE_DENSITY = 0.6

type LinePhase = 'idle' | 'scrambling' | 'resolving' | 'done'

interface LineState {
    phase: LinePhase
    resolvedCount: number
}

// --- Language label helpers ---

function createLabel(text: string): HTMLSpanElement {
    const label = document.createElement('span')
    label.className =
        'absolute top-3 right-3 md:top-4 md:right-4 text-xs text-text-faint transition-opacity duration-200 whitespace-normal'
    label.textContent = text
    return label
}

// --- Static fallback for reduced-motion ---

function renderStatic(container: HTMLDivElement, lines: Segment[][]) {
    for (const line of lines) {
        const div = document.createElement('div')
        div.className = 'leading-relaxed h-[1.75em]'
        for (const seg of line) {
            const span = document.createElement('span')
            span.textContent = seg.text
            span.className = seg.className
            div.appendChild(span)
        }
        container.appendChild(div)
    }
}

// --- Component ---

export default function CodeTransform() {
    const containerRef = useRef<HTMLDivElement>(null)
    const rafRef = useRef(0)
    const isVisibleRef = useRef(false)

    useEffect(() => {
        const container = containerRef.current
        if (!container) return

        // Clear any previous DOM from prior effect run (HMR, Strict Mode)
        container.textContent = ''

        // Respect prefers-reduced-motion — show static GQL, no animation
        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            renderStatic(container, GQL_LINES)
            container.appendChild(createLabel('GraphQL'))
            return
        }

        // Language label in top-right corner
        const label = createLabel('GraphQL')

        // Build DOM structure: one div per line, one span per character position
        const lineCount = GQL_LINES.length
        const charSpans: HTMLSpanElement[][] = []
        const maxLens: number[] = []

        for (let i = 0; i < lineCount; i++) {
            const gqlLen = segmentsToString(GQL_LINES[i]).length
            const tsLen = segmentsToString(TS_LINES[i]).length
            const maxLen = Math.max(gqlLen, tsLen)
            maxLens.push(maxLen)

            const lineEl = document.createElement('div')
            lineEl.className = 'leading-relaxed h-[1.75em]'

            const spans: HTMLSpanElement[] = []
            for (let j = 0; j < maxLen; j++) {
                const span = document.createElement('span')
                spans.push(span)
                lineEl.appendChild(span)
            }
            charSpans.push(spans)
            container.appendChild(lineEl)
        }

        container.appendChild(label)

        // --- Mutable animation state (all in closure, not React state) ---
        let direction: 'gql-to-ts' | 'ts-to-gql' = 'gql-to-ts'
        let phase: 'waiting' | 'animating' | 'paused' = 'waiting'
        let phaseStart = 0
        const lineStates: LineState[] = Array.from(
            { length: lineCount },
            () => ({
                phase: 'idle' as LinePhase,
                resolvedCount: 0,
            }),
        )

        const getSource = () =>
            direction === 'gql-to-ts' ? GQL_LINES : TS_LINES
        const getTarget = () =>
            direction === 'gql-to-ts' ? TS_LINES : GQL_LINES

        /** Write one line's characters directly to the DOM */
        function renderLine(i: number) {
            const spans = charSpans[i]
            const ls = lineStates[i]
            const source = getSource()
            const target = getTarget()
            const srcStr = segmentsToString(source[i])
            const tgtStr = segmentsToString(target[i])
            const len = maxLens[i]

            for (let j = 0; j < len; j++) {
                const span = spans[j]
                switch (ls.phase) {
                    case 'idle': {
                        span.textContent = j < srcStr.length ? srcStr[j] : ' '
                        span.className =
                            j < srcStr.length ? charClassAt(source[i], j) : ''
                        break
                    }
                    case 'scrambling': {
                        const show = Math.random() < SCRAMBLE_DENSITY
                        span.textContent = show ? getScrambleChar() : ' '
                        span.className = show ? 'text-text-faint' : ''
                        break
                    }
                    case 'resolving': {
                        if (j < ls.resolvedCount) {
                            span.textContent =
                                j < tgtStr.length ? tgtStr[j] : ' '
                            span.className =
                                j < tgtStr.length
                                    ? charClassAt(target[i], j)
                                    : ''
                        } else {
                            const show = Math.random() < SCRAMBLE_DENSITY
                            span.textContent = show ? getScrambleChar() : ' '
                            span.className = show ? 'text-text-faint' : ''
                        }
                        break
                    }
                    case 'done': {
                        span.textContent = j < tgtStr.length ? tgtStr[j] : ' '
                        span.className =
                            j < tgtStr.length ? charClassAt(target[i], j) : ''
                        break
                    }
                }
            }
        }

        // Initial render: show GQL statically
        for (let i = 0; i < lineCount; i++) renderLine(i)

        // --- Main animation loop ---
        function tick(now: number) {
            if (!isVisibleRef.current) {
                rafRef.current = requestAnimationFrame(tick)
                return
            }

            if (phaseStart === 0) phaseStart = now
            const elapsed = now - phaseStart

            switch (phase) {
                case 'waiting': {
                    if (elapsed >= INITIAL_DELAY) {
                        phase = 'animating'
                        phaseStart = now
                        label.style.opacity = '0'
                        for (const ls of lineStates) {
                            ls.phase = 'idle'
                            ls.resolvedCount = 0
                        }
                    }
                    break
                }

                case 'animating': {
                    let allDone = true

                    for (let i = 0; i < lineCount; i++) {
                        const ls = lineStates[i]
                        const lineStart = i * LINE_STAGGER

                        if (elapsed < lineStart) {
                            allDone = false
                            continue
                        }

                        const lineElapsed = elapsed - lineStart

                        if (ls.phase === 'idle') {
                            ls.phase = 'scrambling'
                        }

                        if (
                            ls.phase === 'scrambling' &&
                            lineElapsed >= SCRAMBLE_DURATION
                        ) {
                            ls.phase = 'resolving'
                            ls.resolvedCount = 0
                        }

                        if (ls.phase === 'resolving') {
                            const resolveElapsed =
                                lineElapsed - SCRAMBLE_DURATION
                            const progress = Math.min(
                                resolveElapsed / RESOLVE_DURATION,
                                1,
                            )
                            ls.resolvedCount = Math.floor(progress * maxLens[i])
                            if (progress >= 1) ls.phase = 'done'
                        }

                        if (ls.phase !== 'done') allDone = false
                        renderLine(i)
                    }

                    if (allDone) {
                        phase = 'paused'
                        phaseStart = now
                        const targetLabel =
                            direction === 'gql-to-ts' ? 'TypeScript' : 'GraphQL'
                        label.textContent = targetLabel
                        label.style.opacity = '1'
                    }
                    break
                }

                case 'paused': {
                    if (elapsed >= PAUSE_DURATION) {
                        // Flip direction and restart
                        direction =
                            direction === 'gql-to-ts'
                                ? 'ts-to-gql'
                                : 'gql-to-ts'
                        phase = 'animating'
                        phaseStart = now
                        label.style.opacity = '0'
                        for (const ls of lineStates) {
                            ls.phase = 'idle'
                            ls.resolvedCount = 0
                        }
                    }
                    break
                }
            }

            rafRef.current = requestAnimationFrame(tick)
        }

        // --- IntersectionObserver: only animate when visible ---
        const observer = new IntersectionObserver(
            ([entry]) => {
                isVisibleRef.current = entry.isIntersecting
            },
            { threshold: 0.3 },
        )
        observer.observe(container)

        // --- Tab visibility: pause when hidden, avoid time jumps on resume ---
        const onVisibilityChange = () => {
            if (document.hidden) {
                cancelAnimationFrame(rafRef.current)
            } else {
                phaseStart = 0
                rafRef.current = requestAnimationFrame(tick)
            }
        }
        document.addEventListener('visibilitychange', onVisibilityChange)

        // Start the loop
        rafRef.current = requestAnimationFrame(tick)

        return () => {
            cancelAnimationFrame(rafRef.current)
            observer.disconnect()
            document.removeEventListener('visibilitychange', onVisibilityChange)
        }
    }, [])

    return (
        <div
            ref={containerRef}
            className="relative bg-surface-raised rounded-lg p-4 md:p-6 font-mono text-sm md:text-base overflow-x-auto whitespace-pre"
        />
    )
}
