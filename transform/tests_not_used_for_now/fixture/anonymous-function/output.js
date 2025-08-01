import { useJitterScope } from "react-jitter";
export default function() {
    const h = useJitterScope({
        name: "(anonymous)",
        file: "$DIR/tests/fixture/anonymous-function/input.js",
        line: 2,
        offset: 0
    });
    const fieldValues = useFieldValues("name");
    return <div/>;
}
