import { useSyncExternalStore } from "react";
import { engine, type EngineStatus } from "../engine/client";

export function useEngineStatus(): EngineStatus {
  return useSyncExternalStore(engine.subscribe, engine.getStatus);
}
