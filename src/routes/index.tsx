import { component$, useVisibleTask$, useStore, useContextProvider, createContextId } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";
// import { isServer } from '@builder.io/qwik/build'


import { invoke } from "@tauri-apps/api/tauri";
import type { AppState, Display } from "~/cmd";
import { AppContext } from "~/cmd/context";







export default component$(() => {

  const state = useStore<AppState>({
    displayList: [],
    displayItems: []
  })

  useContextProvider(AppContext, state);


  // eslint-disable-next-line qwik/no-use-visible-task
  useVisibleTask$(async () => {
    const res: Display[] = await invoke("display_info")
    state.displayList = res
    const items = res.map((value, index) => {
      return { id: index, label: value.meta.name, value: value.deviceId }
    })
    state.displayItems = items
    console.log(state)
  })

  return (
    <>
      <div>
      </div>
    </>
  );
});

export const head: DocumentHead = {
  title: "Welcome to Qwik",
  meta: [
    {
      name: "description",
      content: "Qwik site description",
    },
  ],
};
