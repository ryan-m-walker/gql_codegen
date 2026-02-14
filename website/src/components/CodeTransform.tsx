import { useRef, useEffect } from 'react'

// --- Syntax-highlighted code segments ---

type Segment = { text: string; className: string }

// prefers-reduced-motion: checked at mount + listened for live changes

// Shorthand helpers to reduce visual noise in line definitions
const kw = (text: string): Segment => ({ text, className: 'text-code-keyword' })
const val = (text: string): Segment => ({ text, className: 'text-code-value' })
const pri = (text: string): Segment => ({
    text,
    className: 'text-text-primary',
})
const sec = (text: string): Segment => ({
    text,
    className: 'text-text-secondary',
})
const blank: Segment[] = [{ text: ' ', className: '' }]

// --- Slide definitions ---

interface Slide {
    label: string
    lines: Segment[][]
}

const SLIDES: Slide[] = [
    // Slide 0: Schema — GraphQL
    {
        label: 'GraphQL',
        lines: [
            [kw('type'), val(' User '), sec('{')],
            [pri('  id'), sec(': '), val('ID!')],
            [pri('  name'), sec(': '), val('String!')],
            [pri('  email'), sec(': '), val('String')],
            [pri('  posts'), sec(': '), val('[Post!]!')],
            [sec('}')],
            blank,
            [kw('enum'), val(' Role '), sec('{')],
            [pri('  ADMIN')],
            [pri('  USER')],
            [pri('  MODERATOR')],
            [sec('}')],
            blank,
            [kw('type'), val(' Post '), sec('{')],
            [pri('  id'), sec(': '), val('ID!')],
            [pri('  title'), sec(': '), val('String!')],
            [pri('  author'), sec(': '), val('User!')],
            [sec('}')],
        ],
    },
    // Slide 1: Schema — TypeScript
    {
        label: 'TypeScript',
        lines: [
            [kw('export'), pri(' '), kw('interface'), val(' User '), sec('{')],
            [
                pri('  '),
                kw('readonly'),
                pri(' id'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('  '),
                kw('readonly'),
                pri(' name'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('  '),
                kw('readonly'),
                pri(' email'),
                sec('?: '),
                val('string'),
                sec(' | '),
                kw('null'),
                sec(';'),
            ],
            [
                pri('  '),
                kw('readonly'),
                pri(' posts'),
                sec(': '),
                val('ReadonlyArray'),
                sec('<Post>'),
                sec(';'),
            ],
            [sec('}')],
            blank,
            [kw('export'), pri(' '), kw('type'), val(' Role '), sec('=')],
            [sec("  | '"), val('ADMIN'), sec("'")],
            [sec("  | '"), val('USER'), sec("'")],
            [sec("  | '"), val('MODERATOR'), sec("'")],
            [sec("  | '"), val('%future added value'), sec("';")],
            blank,
            [kw('export'), pri(' '), kw('interface'), val(' Post '), sec('{')],
            [
                pri('  '),
                kw('readonly'),
                pri(' id'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('  '),
                kw('readonly'),
                pri(' title'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('  '),
                kw('readonly'),
                pri(' author'),
                sec(': '),
                val('User'),
                sec(';'),
            ],
            [sec('}')],
        ],
    },
    // Slide 2: Operation — GraphQL
    {
        label: 'GraphQL',
        lines: [
            [
                kw('query'),
                val(' GetUser'),
                sec('('),
                pri('$id'),
                sec(': '),
                val('ID!'),
                sec(') {'),
            ],
            [
                pri('  user'),
                sec('('),
                pri('id'),
                sec(': '),
                pri('$id'),
                sec(') {'),
            ],
            [pri('    id')],
            [pri('    name')],
            [pri('    email')],
            [pri('    role')],
            [pri('    posts'), sec(' {')],
            [pri('      id')],
            [pri('      title')],
            [sec('    }')],
            [sec('  }')],
            [sec('}')],
        ],
    },
    // Slide 3: Operation — TypeScript
    {
        label: 'TypeScript',
        lines: [
            [
                kw('export'),
                pri(' '),
                kw('type'),
                val(' GetUserQueryVariables '),
                sec('= {'),
            ],
            [
                pri('  '),
                kw('readonly'),
                pri(' id'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [sec('};')],
            blank,
            [
                kw('export'),
                pri(' '),
                kw('type'),
                val(' GetUserQuery '),
                sec('= {'),
            ],
            [pri('  '), kw('readonly'), pri(' user'), sec(': {')],
            [
                pri('    '),
                kw('readonly'),
                pri(' id'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('    '),
                kw('readonly'),
                pri(' name'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('    '),
                kw('readonly'),
                pri(' email'),
                sec('?: '),
                val('string'),
                sec(' | '),
                kw('null'),
                sec(';'),
            ],
            [
                pri('    '),
                kw('readonly'),
                pri(' role'),
                sec(': '),
                val('Role'),
                sec(';'),
            ],
            [
                pri('    '),
                kw('readonly'),
                pri(' posts'),
                sec(': '),
                val('ReadonlyArray'),
                sec('<{'),
            ],
            [
                pri('      '),
                kw('readonly'),
                pri(' id'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [
                pri('      '),
                kw('readonly'),
                pri(' title'),
                sec(': '),
                val('string'),
                sec(';'),
            ],
            [sec('    }'), sec('>'), sec(';')],
            [sec('  }'), sec(' | '), kw('null'), sec(';')],
            [sec('};')],
        ],
    },
]

// --- Scramble character selection ---

const chars = [
    ...Array.from({ length: 20 }, () => '█'),
    ...Array.from({ length: 20 }, () => '░'),
    ...Array.from({ length: 20 }, () => '▒'),
    ...Array.from({ length: 20 }, () => '▓'),
    ...Array.from({ length: 20 }, () => '─'),
    ...Array.from({ length: 20 }, () => '<'),
    ...Array.from({ length: 20 }, () => '>'),
    ...Array.from({ length: 20 }, () => '{'),
    ...Array.from({ length: 20 }, () => '}'),
    ...Array.from({ length: 20 }, () => ';'),
    ...Array.from({ length: 20 }, () => ':'),
    ...Array.from({ length: 20 }, () => '='),
    ...Array.from({ length: 20 }, () => '∷'),
    ...Array.from({ length: 20 }, () => '?'),
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
const LINE_STAGGER = 120
const SCRAMBLE_DURATION = 700
const RESOLVE_DURATION = 500
const PAUSE_DURATION = 3500
const SCRAMBLE_DENSITY = 0.6

type LinePhase = 'idle' | 'scrambling' | 'resolving' | 'done'

interface LineState {
    phase: LinePhase
    resolvedCount: number
}

// --- Language label ---

function createLabel(text: string): HTMLSpanElement {
    const label = document.createElement('span')
    label.className =
        'absolute top-3 right-3 md:top-4 md:right-4 text-xs text-text-muted transition-opacity duration-200 whitespace-normal font-semibold'
    label.textContent = text
    return label
}

// --- Static fallback for reduced-motion ---

function renderStatic(container: HTMLDivElement, slide: Slide) {
    for (const line of slide.lines) {
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

// --- Precompute dimensions across all slides ---

function computeDimensions() {
    const maxLines = Math.max(...SLIDES.map((s) => s.lines.length))
    const maxLens: number[] = []
    for (let i = 0; i < maxLines; i++) {
        let max = 0
        for (const slide of SLIDES) {
            if (i < slide.lines.length) {
                max = Math.max(max, segmentsToString(slide.lines[i]).length)
            }
        }
        maxLens.push(max)
    }
    return { maxLines, maxLens }
}

// --- Component ---

export default function CodeTransform({ className }: { className?: string }) {
    const containerRef = useRef<HTMLDivElement>(null)
    const rafRef = useRef(0)
    const isVisibleRef = useRef(false)

    useEffect(() => {
        const container = containerRef.current
        if (!container) return

        // Clear any previous DOM from prior effect run (HMR, Strict Mode)
        container.textContent = ''

        const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')

        // Respect prefers-reduced-motion — show static slide, no animation
        if (motionQuery.matches) {
            renderStatic(container, SLIDES[0])
            container.appendChild(createLabel(SLIDES[0].label))
            return
        }

        const { maxLines, maxLens } = computeDimensions()
        const label = createLabel(SLIDES[0].label)

        // Build DOM: one div per line, one span per char position
        const charSpans: HTMLSpanElement[][] = []
        for (let i = 0; i < maxLines; i++) {
            const lineEl = document.createElement('div')
            lineEl.className = 'leading-relaxed h-[1.75em]'

            const spans: HTMLSpanElement[] = []
            for (let j = 0; j < maxLens[i]; j++) {
                const span = document.createElement('span')
                spans.push(span)
                lineEl.appendChild(span)
            }
            charSpans.push(spans)
            container.appendChild(lineEl)
        }
        container.appendChild(label)

        // --- Mutable animation state ---
        let slideIndex = 0
        let phase: 'waiting' | 'animating' | 'paused' = 'waiting'
        let phaseStart = 0
        const lineStates: LineState[] = Array.from(
            { length: maxLines },
            () => ({ phase: 'idle' as LinePhase, resolvedCount: 0 }),
        )

        function getSourceSlide() {
            return SLIDES[slideIndex]
        }
        function getTargetSlide() {
            return SLIDES[(slideIndex + 1) % SLIDES.length]
        }

        function renderLine(i: number) {
            const spans = charSpans[i]
            const ls = lineStates[i]
            const source = getSourceSlide().lines
            const target = getTargetSlide().lines
            const srcSegs = i < source.length ? source[i] : []
            const tgtSegs = i < target.length ? target[i] : []
            const srcStr = segmentsToString(srcSegs)
            const tgtStr = segmentsToString(tgtSegs)
            const len = maxLens[i]

            for (let j = 0; j < len; j++) {
                const span = spans[j]
                switch (ls.phase) {
                    case 'idle': {
                        span.textContent = j < srcStr.length ? srcStr[j] : ' '
                        span.className =
                            j < srcStr.length ? charClassAt(srcSegs, j) : ''
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
                                j < tgtStr.length ? charClassAt(tgtSegs, j) : ''
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
                            j < tgtStr.length ? charClassAt(tgtSegs, j) : ''
                        break
                    }
                }
            }
        }

        // Initial render: show first slide statically
        for (let i = 0; i < maxLines; i++) renderLine(i)

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

                    for (let i = 0; i < maxLines; i++) {
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
                        label.textContent = getTargetSlide().label
                        label.style.opacity = '1'
                    }
                    break
                }

                case 'paused': {
                    if (elapsed >= PAUSE_DURATION) {
                        // Advance to next slide
                        slideIndex = (slideIndex + 1) % SLIDES.length
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

        // --- Live reduced-motion listener (e.g. DevTools toggle) ---
        const onMotionChange = (e: MediaQueryListEvent) => {
            if (e.matches) {
                cancelAnimationFrame(rafRef.current)
                container.textContent = ''
                renderStatic(container, SLIDES[slideIndex])
                container.appendChild(createLabel(SLIDES[slideIndex].label))
            }
        }
        motionQuery.addEventListener('change', onMotionChange)

        // --- IntersectionObserver ---
        const observer = new IntersectionObserver(
            ([entry]) => {
                isVisibleRef.current = entry.isIntersecting
            },
            { threshold: 0.3 },
        )
        observer.observe(container)

        // --- Tab visibility ---
        const onVisibilityChange = () => {
            if (document.hidden) {
                cancelAnimationFrame(rafRef.current)
            } else {
                phaseStart = 0
                rafRef.current = requestAnimationFrame(tick)
            }
        }
        document.addEventListener('visibilitychange', onVisibilityChange)

        rafRef.current = requestAnimationFrame(tick)

        return () => {
            cancelAnimationFrame(rafRef.current)
            observer.disconnect()
            document.removeEventListener('visibilitychange', onVisibilityChange)
            motionQuery.removeEventListener('change', onMotionChange)
        }
    }, [])

    return (
        <div
            ref={containerRef}
            className={
                className ??
                'relative bg-surface-raised rounded-lg p-4 md:p-6 font-mono text-sm md:text-base overflow-x-auto whitespace-pre'
            }
        />
    )
}
