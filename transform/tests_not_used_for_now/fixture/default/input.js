import { useState } from "react";

export default function UserForm() {
  const [count, setCount] = useState(0);
  const nameFieldValues = useFieldValues("name");
  const addressFieldValues = useFieldValues("address");

  return <div />;
}
