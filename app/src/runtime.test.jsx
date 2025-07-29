import { render, fireEvent, screen } from '@testing-library/react'
import { describe, it, expect, vi, afterEach } from 'vitest'
import { useState } from 'react'
import { useJitterScope } from '../runtime.js'

function useObj(n) {
  return { n }
}

function ConstantComp({ cb }) {
  globalThis.reactJitter = { onHookChange: cb }
  const h = useJitterScope('Const')
  const meta = { id: 'x', file: 'Const.jsx', hook: 'useObj', line: 1, offset: 0 }
  const val = (h.s('x', meta), h.e('x', useObj(1), meta))
  return <span>{val.n}</span>
}

function ChangingComp({ cb }) {
  globalThis.reactJitter = { onHookChange: cb }
  const h = useJitterScope('Change')
  const [n, setN] = useState(0)
  const meta = { id: 'y', file: 'Change.jsx', hook: 'useObj', line: 1, offset: 0 }
  const val = (h.s('y', meta), h.e('y', useObj(n), meta))
  return <button onClick={() => setN(c => c + 1)}>{val.n}</button>
}

describe('runtime', () => {
  afterEach(() => {
    delete globalThis.reactJitter
  })

  it('does nothing when value stable', () => {
    const cb = vi.fn()
    render(<ConstantComp cb={cb} />)
    expect(cb).not.toHaveBeenCalled()
  })

  it('reports change when value updates', () => {
    const cb = vi.fn()
    render(<ChangingComp cb={cb} />)
    fireEvent.click(screen.getByRole('button'))
    expect(cb).toHaveBeenCalled()
    const report = cb.mock.calls[0][0]
    expect(report.unstable).toBe(true)
    expect(report.changedKeys).toContain('n')
  })
})
