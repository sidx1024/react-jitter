import React from 'react';
import { createElement } from 'react';

export function MyComponent() {
  return React.createElement('div', { className: 'my-class' }, 'Hello');
}

export function AnotherComponent() {
  return createElement('div', { className: 'my-class' }, 'Hello');
}
