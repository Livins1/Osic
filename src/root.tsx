import { component$ } from "@builder.io/qwik";
import {
  QwikCityProvider,
  RouterOutlet,
  ServiceWorkerRegister,
} from "@builder.io/qwik-city";
import { RouterHead } from "./components/router-head/router-head";

import "./global.css";

export default component$(() => {
  /**
   * The root of a QwikCity site always start with the <QwikCityProvider> component,
   * immediately followed by the document's <head> and <body>.
   *
   * Don't remove the `<head>` and `<body>` elements.
   */


  // https://tailwindui.com/components/application-ui/elements/dropdowns


  return (
    <QwikCityProvider>
      <head>
        <meta charSet="utf-8" />
        <link rel="manifest" href="/manifest.json" />
        <RouterHead />
        <ServiceWorkerRegister />
      </head>
      <body lang="en" >
        <div class="bg-black h-full" style="overflow:hidden: padding-right: 0px;">
          <div class="relative flex flex-col bg-white/5">
            <div class="absolute inset-[33%] block rounded-full bg-white/20 blur-2xl">
            </div>
            <div class="relative flex min-h-screen w-full flex-col p-3 ">
              <RouterOutlet />
            </div>
          </div>
        </div>
      </body>
    </QwikCityProvider>
  );
});
