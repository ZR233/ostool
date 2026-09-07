import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listBoards = vi.fn();
const listSessions = vi.fn();
const listLoaderDevices = vi.fn();
const listVirtualDevices = vi.fn();
const createVirtualDevice = vi.fn();
const deleteVirtualDevice = vi.fn();
const deleteBoard = vi.fn();

vi.mock("vue-router", () => ({
  RouterLink: {
    props: ["to"],
    template: "<a><slot /></a>",
  },
}));

vi.mock("@/api/client", () => ({
  api: {
    listBoards,
    listSessions,
    listLoaderDevices,
    listVirtualDevices,
    createVirtualDevice,
    deleteVirtualDevice,
    deleteBoard,
  },
}));

vi.mock("@/stores/ui", () => ({
  useUiStore: () => ({ clearMessages: vi.fn(), setError: vi.fn(), setSuccess: vi.fn() }),
}));

describe("BoardsView loader discovery", () => {
  beforeEach(() => {
    listBoards.mockReset().mockResolvedValue([]);
    listSessions.mockReset().mockResolvedValue({ sessions: [] });
    listLoaderDevices.mockReset().mockResolvedValue([
      {
        mac_address: "02:00:00:00:00:01",
        current_mac_address: "02:00:00:00:00:01",
        ip_address: "10.77.0.2",
        arch: "x86_64",
        loader_version: "0.2.0",
        hardware: {
          manufacturer: "QEMU",
          product: "Standard PC",
          version: "Q35",
          serial: "virtual-1",
        },
        last_seen_at: "2026-09-04T08:00:00Z",
        online: true,
        conflict: false,
        bound_board_id: null,
        current_registration_id: "registration-1",
      },
      {
        mac_address: "02:00:00:00:00:02",
        current_mac_address: "02:00:00:00:00:02",
        ip_address: "10.77.0.3",
        arch: "x86_64",
        loader_version: "0.2.0",
        hardware: { manufacturer: null, product: null, version: null, serial: null },
        last_seen_at: "2026-09-04T08:00:00Z",
        online: true,
        conflict: false,
        bound_board_id: "configured-board",
        current_registration_id: "registration-2",
      },
    ]);
    listVirtualDevices.mockReset().mockResolvedValue({ enabled: false, devices: [] });
    createVirtualDevice.mockReset();
    deleteVirtualDevice.mockReset();
  });

  it("shows hardware for unbound devices and hides configured devices from creation", async () => {
    const BoardsView = (await import("./BoardsView.vue")).default;
    const wrapper = mount(BoardsView);
    await flushPromises();

    expect(wrapper.text()).toContain("02:00:00:00:00:01");
    expect(wrapper.text()).toContain("QEMU Standard PC Q35");
    expect(wrapper.text()).toContain("virtual-1");
    expect(wrapper.text()).not.toContain("02:00:00:00:00:02");
    expect(wrapper.findAll("a").filter((link) => link.text() === "创建配置")).toHaveLength(1);

    wrapper.unmount();
  });
});
