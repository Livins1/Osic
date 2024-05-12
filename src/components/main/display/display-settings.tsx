import { component$, useComputed$, useContext } from "@builder.io/qwik";

import { AppContextId } from "~/cmd/context";


export const displaySettingsProps = {


}

export const Settings = component$(() => {
    const state = useContext(AppContextId)

    const displayInfo = useComputed$(() => {
        return state.displayList.at(state.selectdDisplayIndex) ?? null
    })


    return (
        <>
            <div class=""></div>
        </>
    )

})