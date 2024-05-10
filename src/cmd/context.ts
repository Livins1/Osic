import { createContextId } from "@builder.io/qwik";
import type { AppState } from ".";



export const AppContextId = createContextId<AppState>('ctx.app');