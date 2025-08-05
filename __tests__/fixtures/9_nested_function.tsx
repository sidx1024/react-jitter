import { useState } from 'react';

export function NestedFunction() {
  const [count, setCount] = useState(0);

  function handleClick() {
    setCount(count + 1);
  }

  return (
    <button type="button" onClick={handleClick}>
      Click me
    </button>
  );
}
