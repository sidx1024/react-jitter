import { useState } from "react";
import { useJitterScope } from "react-jitter";

export default function UserForm() {
  const h = useJitterScope({
    name: "UserForm",
    file: "$DIR/tests/fixture/default/input.js",
    line: 4,
    offset: 0,
  });
  const [count, setCount] = useState(0);

  const nameFieldValues =
    (h.s(),
    h.e(useFieldValues("name"), {
      file: "$DIR/tests/fixture/default/input.js",
      hook: "useFieldValues",
      line: 3,
      offset: 28,
    }));

  const addressFieldValues =
    (h.s(),
    h.e(useFieldValues("address"), {
      file: "$DIR/tests/fixture/default/input.js",
      hook: "useFieldValues",
      line: 4,
      offset: 31,
    }));

  return <div />;
}
