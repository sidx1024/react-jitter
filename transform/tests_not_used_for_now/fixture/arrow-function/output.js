import { useJitterScope } from "react-jitter";
export const useAddressField = ()=>{
    const h = useJitterScope("useAddressField");
    return h.s(), h.e(useFieldValues("address"), {
        file: "useAddressField.tsx",
        hook: "useFieldValues",
        line: 1,
        offset: 50
    });
};

