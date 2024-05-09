import { createContextId } from "@builder.io/qwik";
import type { AppState } from ".";



export const AppContext = createContextId<AppState>('ctx.app');