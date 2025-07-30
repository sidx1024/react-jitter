import { useState } from "react";

export default function UserForm(): JSX.Element {
  const [count, setCount] = useState<number>(0);
  const nameFieldValues = useFieldValues<string>("name");
  const addressFieldValues = useFieldValues<string>("address");

  return <div />;
}
