import { render, fireEvent, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import App from './App'

describe('App counter', () => {
  it('increments when clicked', () => {
    render(<App />)
    const btn = screen.getByRole('button')
    fireEvent.click(btn)
    expect(btn.textContent).toContain('1')
  })
})
