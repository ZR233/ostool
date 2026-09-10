import { describe, it, expect, vi } from "vitest";
import { AdminStore } from "./events";
import type { BoardConfig } from "@/types/api";
describe("management event projection", () => {
  it("reconciles rows by ID, removes deleted rows, and preserves unchanged references", () => {
    const store = new AdminStore();
    const a = { id: "a", board_type: "arm" } as BoardConfig,
      b = { id: "b" } as BoardConfig;
    store.apply({
      epoch: "e",
      revision: 1,
      kind: "snapshot",
      data: { boards: [a, b] },
    });
    const original = store.get("boards")!;
    store.apply({
      epoch: "e",
      revision: 2,
      kind: "update",
      data: { boards: [{ ...a }] },
    });
    expect(store.get("boards")).toHaveLength(1);
    expect(store.get("boards")![0]).toBe(original[0]);
    store.apply({
      epoch: "e",
      revision: 1,
      kind: "update",
      data: { boards: [] },
    });
    expect(store.get("boards")).toHaveLength(1);
  });
  it("resets the subscription on a revision gap without clearing displayed data", () => {
    const store = new AdminStore();
    store.apply({
      epoch: "e",
      revision: 1,
      kind: "snapshot",
      data: { boards: [] },
    });
    const restart = vi.spyOn(store, "restart").mockImplementation(() => {});
    store.apply({
      epoch: "e",
      revision: 3,
      kind: "update",
      data: { boards: [{ id: "lost" } as BoardConfig] },
    });
    expect(restart).toHaveBeenCalledOnce();
    expect(store.get("boards")).toEqual([]);
  });
  it("recovers with a new epoch snapshot and exposes per-topic errors without deleting last good data", () => {
    const store = new AdminStore();
    store.apply({
      epoch: "e",
      revision: 4,
      kind: "snapshot",
      data: { boards: [] },
    });
    store.apply({
      epoch: "new",
      revision: 1,
      kind: "snapshot",
      data: { boards: [{ id: "new" } as BoardConfig] },
    });
    store.apply({
      epoch: "new",
      revision: 2,
      kind: "update",
      data: { boards: { error: "unavailable" } as never },
    });
    expect(store.get("boards")![0].id).toBe("new");
    expect(store.getErrors().boards).toBe("unavailable");
    store.apply({
      epoch: "new",
      revision: 3,
      kind: "update",
      data: { boards: [] },
    });
    expect(store.getErrors()).toEqual({});
  });
});
