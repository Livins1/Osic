import { component$, useStore, useContextProvider, useVisibleTask$, $ } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";


import { invoke } from "@tauri-apps/api/tauri";
import type { AppState, Display } from "~/cmd";
import { AppContextId } from "~/cmd/context";
import { DisplaySelector } from "~/components/main/display/display-selector";



export default component$(() => {

  const state = useStore<AppState>({
    displayList: [],
    displayItems: [],
    selectdDisplayIndex: 0,
  })

  useContextProvider(AppContextId, state);

  const getItemList = $(async () => {
    const res: Display[] = await invoke("display_info")
    state.displayList = [...res]
    state.displayItems = [...res.map((value, index) => {
      return { id: index, label: value.meta.name, value: value.deviceId }
    })]
  })


  useVisibleTask$(async () => {
    getItemList()
  })


  return (
    <div class="">
      <DisplaySelector></DisplaySelector>
    </div>
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
